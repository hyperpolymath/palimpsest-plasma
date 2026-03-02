# SPDX-License-Identifier: PMPL-1.0-or-later
# K9 Self-Validating Freelance Journalist Contract
# NUJ-specific validation for freelance journalism contracts

{
  pedigree = {
    name = "freelance-journalist-contract",
    version = "1.0.0",
    description = "Self-validating freelance journalism contract with NUJ ethics compliance",
    magic = "K9!",
    mime_type = "application/vnd.k9+nickel",
    author = "Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>",
  },

  target = {
    os = ["linux", "darwin", "freebsd"],
    edge = {
      container = true,
      wasm = false,
    },
    memory_min_mb = 10,
  },

  security = {
    trust_level = 'Hunt,  # Requires signature for deployment

    permissions = {
      filesystem = { read = true, write = false },
      network = { outbound = false, inbound = false },
      subprocess = { spawn = true, shell = false },
    },
  },

  validation = {
    schema = "../schemas/nuj-code-of-ethics.a2ml",

    # Critical clauses for freelance journalists
    required_clauses = [
      # NUJ Code of Ethics (MUST)
      "source-protection",
      "editorial-independence",
      "right-of-reply",
      "copyright-retention",
      "truth-accuracy",
      "fairness",
      "privacy-harassment",

      # Freelancer-specific (MUST)
      "payment-terms",
      "expenses-reimbursement",
      "termination-notice",
      "ip-ownership",

      # Anti-exploitation (MUST)
      "no-free-trials",
      "no-spec-work",
      "kill-fee-provision",
    ],

    # Recommended clauses (SHOULD)
    recommended_clauses = [
      "late-payment-penalty",
      "portable-benefits",
      "equipment-allowance",
      "training-budget",
      "mental-health-support",
    ],

    checksum | optional = null,
    signature | optional = null,
    signed_by | optional = null,
  },

  contracts = {
    # Payment terms: Must specify NET 30 or better
    payment_terms_reasonable = fun contract =>
      let net_days = contract.payment.terms.net_days in
      std.contract.from_predicate (fun d => d <= 30),

    # Late payment penalty: Minimum 5% after due date
    late_penalty_minimum = fun contract =>
      let penalty_pct = contract.payment.late_penalty_percent in
      std.contract.from_predicate (fun p => p >= 5.0),

    # Kill fee: Minimum 50% if article killed
    kill_fee_minimum = fun contract =>
      let kill_fee_pct = contract.payment.kill_fee_percent in
      std.contract.from_predicate (fun p => p >= 50.0),

    # Copyright: Freelancer retains or has first publication rights only
    copyright_freelancer_friendly = fun contract =>
      let ownership = contract.ip_rights.copyright_ownership in
      std.contract.from_predicate (fun o =>
        o == "freelancer" || o == "first-publication-only"
      ),

    # Day rate: Must meet NUJ minimum
    day_rate_nuj_minimum = fun contract =>
      let rate = contract.payment.day_rate_gbp in
      let nuj_min_2025 = 350.00 in  # NUJ Freelance Fees Guide 2025
      std.contract.from_predicate (fun r => r >= nuj_min_2025),
  },

  recipes = {
    install = m%"
      echo "✅ Freelance journalist contract ready"
      echo "   Payload: ${self.payload_file}"
      echo "   NUJ Schema: ${self.validation.schema}"
      echo ""
      echo "🔍 Key checks:"
      echo "   - Source protection guaranteed"
      echo "   - Editorial independence"
      echo "   - Fair payment terms (NET 30 max)"
      echo "   - Copyright retention"
      echo "   - Kill fee provision"
    "%,

    validate = m%"
      set -e
      echo "🔍 Validating freelance contract (NUJ standards)..."

      # Check NUJ-specific clauses
      union-policy-parser validate ${self.payload_file} \
        --schema ${self.validation.schema} \
        --mode attested \
        --union nuj \
        --required-clauses "${std.string.join "," self.validation.required_clauses}"

      # Check payment terms
      echo ""
      echo "💰 Payment terms check:"
      union-policy-parser check-clause ${self.payload_file} \
        --clause "payment-terms.net-days" \
        --max 30 \
        --error-if-exceeds

      union-policy-parser check-clause ${self.payload_file} \
        --clause "payment.late-penalty-percent" \
        --min 5.0 \
        --warn-if-below

      # Check copyright
      echo ""
      echo "©️  Copyright check:"
      union-policy-parser check-clause ${self.payload_file} \
        --clause "ip-rights.copyright-ownership" \
        --allowed "freelancer,first-publication-only" \
        --error-if-not

      echo ""
      echo "✅ Freelance contract validates against NUJ standards"
    "%,

    audit = m%"
      set -e
      echo "📊 NUJ Freelance Contract Audit"
      echo "================================"

      union-policy-parser audit ${self.payload_file} \
        --schema ${self.validation.schema} \
        --union nuj \
        --check-exploitative-clauses \
        --check-nuj-minimums \
        --output /tmp/freelance-audit-$(date +%Y%m%d-%H%M%S).json

      echo ""
      echo "Audit saved to /tmp/"
    "%,

    # NUJ-specific: Check for exploitative clauses
    check_exploitation = m%"
      set -e
      echo "⚠️  Checking for exploitative clauses..."

      # Red flags for freelancers
      red_flags=$(union-policy-parser scan-red-flags ${self.payload_file} \
        --patterns \
          "all rights" \
          "work for hire" \
          "perpetual license" \
          "free trial" \
          "spec work" \
          "unpaid" \
          "no kill fee" \
          "payment on publication" \
        --case-insensitive)

      if [ -n "$red_flags" ]; then
        echo "❌ EXPLOITATIVE CLAUSES DETECTED:"
        echo "$red_flags"
        echo ""
        echo "⚠️  DO NOT SIGN THIS CONTRACT"
        echo "   Contact NUJ rep: freelance@nuj.org.uk"
        exit 1
      else
        echo "✅ No obvious exploitative clauses found"
      fi
    "%,

    # Auto-negotiate: Suggest improvements
    negotiate = m%"
      set -e
      echo "💼 Contract Negotiation Suggestions"
      echo "===================================="

      union-policy-parser negotiate ${self.payload_file} \
        --schema ${self.validation.schema} \
        --union nuj \
        --output /tmp/negotiation-points-$(date +%Y%m%d-%H%M%S).md

      echo ""
      echo "📄 Negotiation points saved to /tmp/"
      echo ""
      echo "Common improvements for freelancers:"
      echo "  1. Reduce NET days from 60 → 30"
      echo "  2. Add late payment penalty (5-10%)"
      echo "  3. Increase kill fee to 50-100%"
      echo "  4. Retain copyright (or first-pub-only)"
      echo "  5. Add expenses reimbursement clause"
    "%,

    sign = m%"
      set -e

      # Validate first
      just -f $0 validate

      # Check exploitation
      just -f $0 check_exploitation

      echo ""
      echo "🔏 Signing freelance contract..."

      # NUJ rep signs to certify compliance
      if [ ! -f ~/.ssh/nuj-freelance-rep-ed25519 ]; then
        echo "❌ NUJ freelance rep key not found"
        exit 1
      fi

      checksum=$(sha256sum ${self.payload_file} | cut -d' ' -f1)
      signature=$(echo -n "$checksum" | ssh-keygen -Y sign -f ~/.ssh/nuj-freelance-rep-ed25519 -n freelance-contract | base64 -w0)

      echo "✅ Contract certified by NUJ freelance rep"
      echo "   Checksum:  $checksum"
      echo "   Signature: $signature"
    "%,

    deploy = m%"
      set -e

      # Full validation pipeline
      just -f $0 validate
      just -f $0 check_exploitation

      if [ -n "${self.validation.signature}" ]; then
        just -f $0 verify
      fi

      deploy_dir="/var/nuj-contracts/freelance"
      mkdir -p "$deploy_dir"

      contract_id=$(basename ${self.payload_file} .a2ml)
      timestamp=$(date +%Y%m%d-%H%M%S)

      cp ${self.payload_file} "$deploy_dir/${contract_id}-${timestamp}.a2ml"

      echo "✅ Freelance contract deployed"
      echo "   Safe to sign and return to employer"
    "%,

    grievance = m%"
      set -e

      echo "⚖️  Checking NUJ Code violations..."

      if ! union-policy-parser validate ${self.payload_file} --schema ${self.validation.schema} 2>&1; then
        echo "❌ NUJ Code violations detected"

        union-policy-parser grievance ${self.payload_file} \
          --schema ${self.validation.schema} \
          --union nuj \
          --template ../templates/nuj-freelance-grievance.md \
          --output /tmp/grievance-freelance-$(date +%Y%m%d-%H%M%S).md

        echo ""
        echo "📧 Contact NUJ Freelance Office:"
        echo "   Email: freelance@nuj.org.uk"
        echo "   Phone: +44 (0)20 7843 3700"
      fi
    "%,
  },

  payload_file = "../examples/freelance-journalist-contract.a2ml",
}
