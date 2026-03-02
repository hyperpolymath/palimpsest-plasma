# SPDX-License-Identifier: PMPL-1.0-or-later
# K9 Self-Validating Editorial Policy
# For newsrooms/media organizations implementing NUJ ethical standards

{
  pedigree = {
    name = "nuj-editorial-policy",
    version = "1.0.0",
    description = "Self-validating editorial policy with NUJ Code compliance",
    magic = "K9!",
    mime_type = "application/vnd.k9+nickel",
    author = "Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>",
  },

  target = {
    os = ["linux", "darwin", "freebsd"],
    edge = {
      container = true,
      wasm = true,  # Can run in browser for public transparency
    },
    memory_min_mb = 10,
  },

  security = {
    trust_level = 'Yard,  # Public-facing, read-only validation

    permissions = {
      filesystem = { read = true, write = false },
      network = { outbound = false, inbound = false },
      subprocess = { spawn = false, shell = false },  # Pure validation only
    },
  },

  validation = {
    schema = "../schemas/nuj-code-of-ethics.a2ml",

    # Core NUJ editorial principles (MUST)
    required_clauses = [
      # Truth & Accuracy
      "corrections-policy",
      "fact-checking-process",
      "sources-verification",

      # Independence
      "editorial-independence-guarantee",
      "conflicts-of-interest-policy",
      "gifts-policy",
      "advertiser-separation",

      # Fairness
      "right-of-reply-procedure",
      "complaints-handling",
      "impartiality-standards",

      # Privacy
      "privacy-respect-policy",
      "public-interest-criteria",
      "vulnerable-subjects-protection",

      # Accountability
      "transparency-about-errors",
      "public-editor-contact",
      "editorial-standards-contact",

      # Source Protection
      "source-protection-guarantee",
      "secure-communications-policy",

      # Discrimination
      "anti-discrimination-policy",
      "diversity-style-guide",
      "hate-speech-policy",

      # Plagiarism
      "attribution-requirements",
      "copyright-compliance",
    ],

    # Best practices (SHOULD)
    recommended_clauses = [
      "transparency-funding-sources",
      "ai-disclosure-policy",
      "diversity-reporting-standards",
      "accessibility-standards",
      "environmental-policy",
    ],

    checksum | optional = null,
    signature | optional = null,
    signed_by | optional = null,  # Newsroom editor's signature
  },

  contracts = {
    # Corrections must be prominent (not buried)
    corrections_prominent = fun policy =>
      let prominence = policy.corrections.prominence_level in
      std.contract.from_predicate (fun p =>
        p == "same-position" || p == "front-page" || p == "top-of-site"
      ),

    # Right of reply must be timely (≤72 hours)
    reply_timely = fun policy =>
      let hours = policy.fairness.right_of_reply_hours_max in
      std.contract.from_predicate (fun h => h <= 72),

    # Source protection: no forced disclosure
    source_protection_absolute = fun policy =>
      let forced_disclosure = policy.source_protection.forced_disclosure_allowed in
      std.contract.from_predicate (fun f => f == false),

    # AI disclosure: Must label AI-generated content
    ai_disclosure_required = fun policy =>
      let disclosure = policy.transparency.ai_disclosure_required in
      std.contract.from_predicate (fun d => d == true),

    # Advertiser separation: Clear firewall
    advertiser_firewall = fun policy =>
      let firewall = policy.independence.advertiser_editorial_firewall in
      std.contract.from_predicate (fun f => f == true),
  },

  recipes = {
    install = m%"
      echo "✅ NUJ Editorial Policy loaded"
      echo "   Payload: ${self.payload_file}"
      echo "   Schema:  ${self.validation.schema}"
      echo ""
      echo "📰 Editorial standards enforced:"
      echo "   ✓ Truth & Accuracy (corrections, fact-checking)"
      echo "   ✓ Editorial Independence (advertiser firewall)"
      echo "   ✓ Fairness (right of reply)"
      echo "   ✓ Privacy (public interest test)"
      echo "   ✓ Source Protection (no forced disclosure)"
      echo "   ✓ Anti-Discrimination"
    "%,

    validate = m%"
      set -e
      echo "🔍 Validating editorial policy (NUJ Code)..."

      union-policy-parser validate ${self.payload_file} \
        --schema ${self.validation.schema} \
        --mode attested \
        --union nuj \
        --policy-type editorial \
        --required-clauses "${std.string.join "," self.validation.required_clauses}"

      # Check critical safeguards
      echo ""
      echo "🛡️  Checking critical safeguards:"

      # Source protection
      union-policy-parser check-clause ${self.payload_file} \
        --clause "source-protection.forced-disclosure-allowed" \
        --expected false \
        --error-if-not

      # Advertiser firewall
      union-policy-parser check-clause ${self.payload_file} \
        --clause "independence.advertiser-editorial-firewall" \
        --expected true \
        --error-if-not

      # Right of reply timeliness
      union-policy-parser check-clause ${self.payload_file} \
        --clause "fairness.right-of-reply-hours-max" \
        --max 72 \
        --warn-if-exceeds

      echo ""
      echo "✅ Editorial policy complies with NUJ Code"
    "%,

    audit = m%"
      set -e
      echo "📊 NUJ Editorial Policy Audit"
      echo "=============================="
      echo ""

      union-policy-parser audit ${self.payload_file} \
        --schema ${self.validation.schema} \
        --union nuj \
        --check-nuj-code-alignment \
        --check-ipso-compliance \
        --check-ofcom-compliance \
        --output /tmp/editorial-audit-$(date +%Y%m%d-%H%M%S).json

      echo ""
      echo "Audit covers:"
      echo "  - NUJ Code of Conduct compliance"
      echo "  - IPSO Editors' Code alignment"
      echo "  - Ofcom Broadcasting Code (if applicable)"
      echo ""
      echo "Report saved to /tmp/"
    "%,

    # Public transparency: Publish policy for readers
    publish = m%"
      set -e

      # Validate first
      just -f $0 validate

      echo ""
      echo "📢 Publishing editorial policy for public transparency..."

      # Render to HTML
      union-policy-parser render ${self.payload_file} \
        --format html \
        --template ../templates/editorial-policy-public.html \
        --output /var/www/about/editorial-standards.html

      # Render to plain text
      union-policy-parser render ${self.payload_file} \
        --format markdown \
        --output /var/www/about/editorial-standards.md

      echo ""
      echo "✅ Editorial policy published:"
      echo "   HTML: /var/www/about/editorial-standards.html"
      echo "   MD:   /var/www/about/editorial-standards.md"
      echo ""
      echo "🔗 Make accessible at:"
      echo "   https://example.com/about/editorial-standards"
    "%,

    # AI ethics check: Ensure AI disclosure
    check_ai_ethics = m%"
      set -e
      echo "🤖 Checking AI ethics compliance..."

      # Does policy require AI disclosure?
      ai_disclosure=$(union-policy-parser get-clause ${self.payload_file} \
        --clause "transparency.ai-disclosure-required")

      if [ "$ai_disclosure" != "true" ]; then
        echo "❌ AI disclosure not required - violates NUJ AI Ethics"
        echo ""
        echo "⚠️  NUJ AI Ethics Guide (2024) requires:"
        echo "   - Disclosure of AI-generated content"
        echo "   - Human review for all AI suggestions"
        echo "   - No AI replacement of journalists"
        exit 1
      fi

      # Does policy protect jobs from AI?
      ai_replacement=$(union-policy-parser get-clause ${self.payload_file} \
        --clause "ai-policy.journalist-replacement-allowed")

      if [ "$ai_replacement" == "true" ]; then
        echo "❌ Policy allows AI replacement of journalists"
        echo "   Violates NUJ AI Ethics (job protection)"
        exit 1
      fi

      echo "✅ AI ethics compliant"
    "%,

    # Diversity audit
    check_diversity = m%"
      set -e
      echo "🌍 Checking diversity commitments..."

      union-policy-parser audit-diversity ${self.payload_file} \
        --check-source-diversity \
        --check-representation-standards \
        --check-style-guide-inclusive \
        --output /tmp/diversity-audit-$(date +%Y%m%d-%H%M%S).json

      echo ""
      echo "Diversity audit complete (see /tmp/)"
      echo ""
      echo "NUJ Equality Guidelines checklist:"
      echo "  - Gender balance in expert sources"
      echo "  - BAME representation"
      echo "  - LGBTQ+ inclusion"
      echo "  - Disability representation"
      echo "  - Age diversity"
    "%,

    sign = m%"
      set -e

      # Validate first
      just -f $0 validate
      just -f $0 check_ai_ethics
      just -f $0 check_diversity

      echo ""
      echo "🔏 Signing editorial policy..."

      # Editor signs to certify NUJ compliance
      if [ ! -f ~/.ssh/editor-ed25519 ]; then
        echo "❌ Editor signing key not found"
        exit 1
      fi

      checksum=$(sha256sum ${self.payload_file} | cut -d' ' -f1)
      signature=$(echo -n "$checksum" | ssh-keygen -Y sign -f ~/.ssh/editor-ed25519 -n editorial-policy | base64 -w0)

      echo "✅ Editorial policy certified"
      echo "   Signed by: Editor-in-Chief"
      echo "   Checksum:  $checksum"
      echo ""
      echo "⚠️  Update this file with:"
      echo "    validation.checksum = \"$checksum\""
      echo "    validation.signature = \"$signature\""
    "%,

    deploy = m%"
      set -e

      # Full validation pipeline
      just -f $0 validate
      just -f $0 check_ai_ethics
      just -f $0 check_diversity

      # Publish for transparency
      just -f $0 publish

      # Archive
      policy_dir="/var/newsroom/editorial-policies"
      mkdir -p "$policy_dir"

      timestamp=$(date +%Y%m%d-%H%M%S)
      cp ${self.payload_file} "$policy_dir/policy-${timestamp}.a2ml"

      echo ""
      echo "✅ Editorial policy deployed"
      echo "   Staff can access at /var/newsroom/editorial-policies/"
      echo "   Public can access at example.com/about/editorial-standards"
    "%,

    # Staff training: Generate quiz
    training_quiz = m%"
      set -e
      echo "📚 Generating NUJ Code training quiz..."

      union-policy-parser generate-quiz ${self.payload_file} \
        --schema ${self.validation.schema} \
        --questions 20 \
        --format interactive \
        --output /var/newsroom/training/nuj-code-quiz.html

      echo ""
      echo "✅ Training quiz generated"
      echo "   Location: /var/newsroom/training/nuj-code-quiz.html"
      echo ""
      echo "Quiz covers:"
      echo "  - Source protection scenarios"
      echo "  - Right of reply procedures"
      echo "  - Privacy vs public interest"
      echo "  - Conflicts of interest"
      echo "  - Corrections policy"
    "%,
  },

  payload_file = "../examples/nuj-editorial-policy.a2ml",
}
