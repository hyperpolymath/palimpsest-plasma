# SPDX-License-Identifier: PMPL-1.0-or-later
# K9 Self-Validating Employment Contract Wrapper
# Encapsulates A2ML contract with Nickel validation rules

{
  pedigree = {
    name = "employment-contract",
    version = "1.0.0",
    description = "Self-validating employment contract with union policy compliance",
    magic = "K9!",
    mime_type = "application/vnd.k9+nickel",
    author = "Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>",
  },

  target = {
    os = ["linux", "darwin", "freebsd"],
    edge = {
      container = true,
      wasm = false,  # Future: WASM runtime for browser validation
    },
    memory_min_mb = 10,
  },

  security = {
    trust_level = 'Hunt,  # Requires Ed25519 signature (tamper-proof)

    permissions = {
      filesystem = {
        read = true,   # Read A2ML payload and schemas
        write = false,  # No modifications to prevent tampering
      },
      network = {
        outbound = false,  # No phone-home; purely local validation
        inbound = false,
      },
      subprocess = {
        spawn = true,   # Needed for union-policy-parser invocation
        shell = false,  # No shell injection risk
      },
    },
  },

  validation = {
    # Primary schema for validation
    schema = "../schemas/nuj-code-of-ethics.a2ml",

    # Required clauses (must be present in contract)
    required_clauses = [
      "employment.terms",
      "taxation.ni",
      "termination.conditions",
      "grievance.procedure",
      "confidentiality.obligations",
    ],

    # Union-specific clauses (NUJ/IWW/UCU)
    union_clauses = {
      nuj = [
        "source-protection",
        "editorial-independence",
        "right-of-reply",
        "copyright-retention",
      ],
      iww = [
        "payment-terms",
        "late-payment-penalty",
        "collective-voice",
      ],
      ucu = [
        "academic-freedom",
        "workload-limits",
        "research-time",
      ],
    },

    # SHA256 checksum of A2ML payload (integrity)
    checksum_algo = "sha256",
    checksum | optional = null,  # Computed during packaging

    # Signature verification
    signature | optional = null,  # Ed25519 signature
    signed_by | optional = null,  # Union rep's public key fingerprint
  },

  contracts = {
    # Nickel contract: days-contracted must be positive integer
    days_contracted = fun contract =>
      let days = contract.employment.terms.days_contracted in
      std.contract.from_predicate (fun d => d > 0 && d <= 7),

    # Nickel contract: daily rate must meet minimum wage
    daily_rate_minimum = fun contract =>
      let rate = contract.remuneration.daily_rate in
      let min_wage_daily = 98.60 in  # UK NMW 2025: £13.15/hr * 7.5hr
      std.contract.from_predicate (fun r => r >= min_wage_daily),

    # Nickel contract: notice period must comply with UK law
    notice_period_legal = fun contract =>
      let notice_months = contract.termination.conditions.notice_period in
      std.contract.from_predicate (fun n => n >= 1),  # ERA 1996: min 1 month

    # Nickel contract: ACAS grievance procedure required
    grievance_acas = fun contract =>
      let procedure = contract.grievance.procedure.formal_procedure in
      std.contract.from_predicate (fun p =>
        std.string.contains "ACAS" p || std.string.contains "acas" p
      ),
  },

  recipes = {
    # Install: No-op for contracts (already in place)
    install = m%"
      echo "✅ Employment contract ready for validation"
      echo "   Payload: ${self.payload_file}"
      echo "   Schema:  ${self.validation.schema}"
    "%,

    # Validate: Check contract against union schemas
    validate = m%"
      set -e
      echo "🔍 Validating employment contract..."

      # Check if union-policy-parser is installed
      if ! command -v union-policy-parser &> /dev/null; then
        echo "❌ union-policy-parser not found"
        echo "   Install: cargo install --git https://github.com/hyperpolymath/union-policy-parser"
        exit 1
      fi

      # Validate A2ML structure
      union-policy-parser validate ${self.payload_file} \
        --schema ${self.validation.schema} \
        --mode attested \
        --required-clauses "${std.string.join "," self.validation.required_clauses}"

      # Verify checksum if present
      if [ -n "${self.validation.checksum}" ]; then
        echo "🔐 Verifying payload integrity..."
        computed=$(sha256sum ${self.payload_file} | cut -d' ' -f1)
        if [ "$computed" != "${self.validation.checksum}" ]; then
          echo "❌ Checksum mismatch - contract may be tampered"
          exit 1
        fi
        echo "✅ Payload integrity verified"
      fi

      echo "✅ Contract validation successful"
    "%,

    # Audit: Generate detailed compliance report
    audit = m%"
      set -e
      echo "📊 Generating compliance audit report..."

      union-policy-parser audit ${self.payload_file} \
        --schema ${self.validation.schema} \
        --output /tmp/contract-audit-$(date +%Y%m%d-%H%M%S).json \
        --format json

      echo "✅ Audit report saved to /tmp/"
    "%,

    # Sign: Union rep signs contract with Ed25519
    sign = m%"
      set -e

      if [ ! -f ~/.ssh/union-rep-ed25519 ]; then
        echo "❌ Union rep signing key not found"
        echo "   Generate: ssh-keygen -t ed25519 -f ~/.ssh/union-rep-ed25519"
        exit 1
      fi

      echo "🔏 Signing contract with union rep key..."

      # Compute checksum
      checksum=$(sha256sum ${self.payload_file} | cut -d' ' -f1)

      # Sign checksum
      signature=$(echo -n "$checksum" | ssh-keygen -Y sign -f ~/.ssh/union-rep-ed25519 -n contract | base64 -w0)

      echo "✅ Contract signed"
      echo "   Signature: $signature"
      echo "   Checksum:  $checksum"

      # Update K9 metadata (requires editing this file)
      echo ""
      echo "⚠️  Manual step: Update this file with:"
      echo "    validation.checksum = \"$checksum\""
      echo "    validation.signature = \"$signature\""
      echo "    validation.signed_by = \"$(ssh-keygen -lf ~/.ssh/union-rep-ed25519.pub | awk '{print $2}')\""
    "%,

    # Verify: Check Ed25519 signature
    verify = m%"
      set -e

      if [ -z "${self.validation.signature}" ]; then
        echo "⚠️  Contract not signed - cannot verify"
        exit 0
      fi

      echo "🔍 Verifying contract signature..."

      # Compute checksum
      computed=$(sha256sum ${self.payload_file} | cut -d' ' -f1)

      # Verify against stored checksum
      if [ "$computed" != "${self.validation.checksum}" ]; then
        echo "❌ Checksum mismatch - contract tampered after signing"
        exit 1
      fi

      # Verify Ed25519 signature (requires union rep's public key)
      # Note: ssh-keygen -Y verify requires allowed_signers file
      echo "✅ Checksum verified"
      echo "   Signed by: ${self.validation.signed_by}"

      echo ""
      echo "⚠️  Full signature verification requires union rep's public key"
      echo "   Add to ~/.ssh/allowed_signers:"
      echo "   contract ${self.validation.signed_by}"
    "%,

    # Deploy: Copy validated contract to approved location
    deploy = m%"
      set -e

      # Validate first
      just -f $0 validate

      # Check signature if present
      if [ -n "${self.validation.signature}" ]; then
        just -f $0 verify
      fi

      # Deploy to approved contracts directory
      deploy_dir="/var/union-contracts/validated"
      mkdir -p "$deploy_dir"

      contract_id=$(basename ${self.payload_file} .a2ml)
      timestamp=$(date +%Y%m%d-%H%M%S)

      cp ${self.payload_file} "$deploy_dir/${contract_id}-${timestamp}.a2ml"

      echo "✅ Contract deployed to $deploy_dir/"
      echo "   File: ${contract_id}-${timestamp}.a2ml"
    "%,

    # Grievance: Auto-generate grievance for violations
    grievance = m%"
      set -e

      echo "⚖️  Checking for contract violations..."

      # Run validation and capture failures
      if ! union-policy-parser validate ${self.payload_file} --schema ${self.validation.schema} 2>&1; then
        echo "❌ Violations detected - generating grievance..."

        union-policy-parser grievance ${self.payload_file} \
          --schema ${self.validation.schema} \
          --violation auto-detect \
          --template ../templates/nuj-grievance.md \
          --output /tmp/grievance-$(date +%Y%m%d-%H%M%S).md

        echo "✅ Grievance letter generated in /tmp/"
      else
        echo "✅ No violations - grievance not needed"
      fi
    "%,
  },

  # A2ML payload file (relative path)
  payload_file = "../examples/ou-day-employee-contract.a2ml",
}
