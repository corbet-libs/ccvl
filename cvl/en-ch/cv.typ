// Editorial rule for the entire CV:
// Maximize information density: every word and syllable must add a relevant signal.
// Each statement should communicate at least one of: outcome, scale, recognizable
// reference, ownership, or relevant method. Prefer Google XYZ phrasing: lead with
// impact, quantify it where possible, then state how it was achieved. Remove filler,
// defensive qualifiers, generic duties, and duplicated meaning. Write concisely and
// confidently without inventing facts; selective emphasis and compression are expected.
// Every bullet must remain exactly one rendered line and use 80–100% of the
// available line width, targeting 90%; the renderer enforces this contract.
// Add signal, never filler, when a bullet is too short.
// If verified content cannot fill that range, identify the missing dimension
// (outcome, metric, scope, reference, ownership, or method) and ask for the
// relevant fact instead of padding or weakening the wording.

#import "/.agent/typst/styles/harvard.typ": document-style, load-style
#import "/.agent/typst/application.typ": cv-contract, last-line-maximum, validate-application
#import "/.agent/typst/line-contract.typ": measured-content-line, measured-lines, wrap-exact
#import "/.agent/typst/profile.typ": localized-profile, profile
#set document(title: "Curriculum Vitae | " + profile.name, author: (profile.name,))

// Page count is selected at compile time: 2 = core, 3 = projects, 4 = competencies.
#let cv-pages = int(sys.inputs.at("cv-pages", default: "4"))
#assert(cv-pages >= 2 and cv-pages <= 4, message: "cv-pages must be 2, 3, or 4")
#let application-path = sys.inputs.at("application", default: "/cvl/en-ch/application.toml")
#let application = toml(application-path)
#validate-application(application, expected-language: "en-CH", require-cv: true)

// Style axis: the explicit `style` input injected by render.rs (resolved from
// options.style) wins; a manual render without it falls back to the record,
// then to the harvard default. Whitespace below comes from that style's TOML
// knobs, never from forked literals.
#let style-input = sys.inputs.at("style", default: "")
#let style-name = if style-input != "" { style-input } else { application.options.at("style", default: "harvard") }
#let style = load-style(style-name)
#show: document-style.with(locale: "en-ch", style: style)

// Style whitespace knobs from the active style's TOML; the element styles
// below consume these instead of forked literals.
#let cv-superheading-outer-spacing = style.cv.superheading_outer_spacing_pt * 1pt
#let cv-compact-heading-spacing = style.cv.compact_heading_spacing_pt * 1pt
#let cv-spacious-heading-spacing = style.cv.spacious_heading_spacing_pt * 1pt
#let cv-entry-spacing = style.cv.entry_spacing_pt * 1pt
#let cv-heading-after = style.cv.heading_after_pt * 1pt
#let cv-subheading-after = style.cv.subheading_after_pt * 1pt
#let cv-bullet-after = style.cv.bullet_after_pt * 1pt
#let cv-competency-heading-after = style.cv.competency_heading_after_pt * 1pt
#let cv-rule-gap = style.cv.rule_gap_pt * 1pt
#let cv-superheading-inner = style.cv.superheading_inner_pt * 1pt
#let cv-header-after = style.header.after_pt * 1pt
#let cv-bullet-indent = style.cv.bullet_indent_pt * 1pt

// Visible CV presentation: every element style and the letterhead live here
// so editors never hunt below .agent/typst. Only page setup, measurement,
// validation, and profile data stay imported.
#let cv-bullet() = box(width: 10.5pt, height: 7.35pt, align(horizon, align(center, polygon(
  fill: rgb("#000000"),
  (0pt, 0pt),
  (4.41pt, 2.75625pt),
  (0pt, 5.5125pt),
))))

// Entry heading (bold). Override size: #cv-h(size: 14pt)[...]
#let cv-h(size: 11pt, min-fill: 15, target-fill: 45, max-fill: 100, t) = measured-content-line(
  "cv.heading",
  "cv-heading",
  text(size: size, weight: "bold", t),
  min-fill,
  target-fill,
  max-fill,
)
#let cv-hu(size: 11pt, min-fill: 60, target-fill: 85, max-fill: 100, t) = {
  set strong(delta: -300)
  measured-content-line(
    "cv.emphasized-heading",
    "cv-emphasized-heading",
    text(size: size, weight: "bold", t),
    min-fill,
    target-fill,
    max-fill,
  )
}
// Entry subheading. Override size: #cv-s(size: 9pt)[...]
#let cv-s(size: 10pt, min-fill: 35, target-fill: 65, max-fill: 100, t) = measured-content-line(
  "cv.subheading",
  "cv-subheading",
  text(size: size, t),
  min-fill,
  target-fill,
  max-fill,
)
// Bullet row. Override indent/gutter: #cv-b(indent: 12pt)[...]
#let cv-b(indent: cv-bullet-indent, gutter: 0pt, min-fill: 80, target-fill: 90, max-fill: 100, t) = grid(
  columns: (indent, 1fr),
  gutter: gutter,
  cv-bullet(), measured-content-line("cv.bullet", "cv-bullet", t, min-fill, target-fill, max-fill),
)

#let cv-entry-gap() = v(cv-entry-spacing)

// Page-level heading for dedicated CV pages such as projects or competencies.
#let cv-superheading(t) = {
  block(width: 100%, breakable: false, inset: (top: cv-superheading-outer-spacing))[
    #set par(spacing: 0pt)
    #line(length: 100%, stroke: 0.5pt + black)
    #v(cv-superheading-inner)
    #align(center, text(size: 17pt, weight: "bold", upper(t)))
    #v(cv-superheading-inner)
    #line(length: 100%, stroke: 0.5pt + black)
  ]
}

// Shared renderer for section headings with symmetric outer spacing.
#let cv-section-heading(spacing, t) = block(breakable: false)[
  #v(spacing)
  #set par(spacing: 0pt)
  #text(size: 12pt, weight: "bold", upper(t))
  #v(cv-rule-gap)
  #line(length: 100%, stroke: 0.5pt + black)
  #v(spacing)
]

// Compact section heading for dense CV pages.
#let cv-compact-heading(t) = cv-section-heading(cv-compact-heading-spacing, t)

// Spacious section heading for dedicated project and competency pages.
#let cv-spacious-heading(t) = cv-section-heading(cv-spacious-heading-spacing, t)

// Keep named CV variants honest: a fourpager must render exactly four pages.
#let assert-page-count(expected) = context {
  let actual = counter(page).final().first()
  assert(actual == expected, message: "CV rendered " + str(actual) + " pages; expected " + str(expected))
}

#let brand(body) = box(body)
#let cv-header() = {
  let localized = localized-profile.at("en-ch")
  let contacts = (
    link("mailto:" + profile.email)[#profile.email],
    if profile.phone-label != none and profile.phone-href != none {
      link(profile.phone-href)[#profile.phone-label]
    },
    profile.location,
    profile.languages,
    localized.nationality-and-permit,
    link(profile.linkedin)[LinkedIn],
    link(profile.website)[Web],
    localized.availability,
  ).filter(item => item != none)

  align(center)[#text(size: 15.75pt, weight: "bold")[#profile.name]]
  v(cv-header-after)
  align(center)[
    #text(size: 9.03pt)[
      #contacts.join([ | ])
    ]
  ]
}
#let cv-pagebreak() = [
  #pagebreak()
  #cv-header()
]

#cv-header()

#block(breakable: false)[
  #cv-compact-heading[Summary]
  // The record holds one flowing paragraph; wrapping to exactly five lines
  // happens here so authors never count breaks by hand.
  #set text(hyphenate: false)
  #layout(size => {
    let fill = cv-contract.summary_fill
    let thin-ok = "allow_thin" in application.cv and application.cv.allow_thin
    let lines = wrap-exact(
      application.cv.summary,
      size.width,
      5,
      "application.cv.summary",
      last-max: last-line-maximum,
    )
    let mapped = range(lines.len()).map(index => {
      let line = lines.at(index)
      (
        text: line,
        min_fill: if thin-ok { 1 } else { fill.minimum },
        target_fill: fill.target,
        max_fill: if index + 1 == lines.len() { last-line-maximum } else { fill.maximum },
      )
    })
    // Pre-measured summary lines render at exact width so an approved
    // closing-line spill stays invisibly in the margin instead of wrapping.
    measured-lines("cv.summary", "cv-summary", mapped, exact-width: true)
  })
]

#block(breakable: false)[
  #cv-compact-heading[Experience]
  // ccvl-station: cenvion
  #cv-h[Infrastructure Investments & Asset Management: #brand[CENVION]]
  #v(cv-heading-after)
  #cv-s[Associate Intern | Infrastructure Investments · Jan 2026 – Mar 2026 (plus freelance work) · Wollerau (CH)]
  #v(cv-subheading-after)
  #cv-b[Introduced #brand[Claude] for investment reporting; embedded GenAI in the team's analysis and reporting workflows]
  #v(cv-bullet-after)
  #cv-b[Developed RAG-based AI search across project and portfolio data; made internal knowledge searchable]
  #v(cv-bullet-after)
  #cv-b[Built Excel project-finance models; analysed cash flows, returns and financing scenarios]
  #cv-entry-gap()
  // ccvl-station: swisscom
  #cv-h[Cloud Strategy & Transformation: #brand[Swisscom Financial Services]]
  #v(cv-heading-after)
  #cv-s[Executive Assistant & Consultant | B2B & Infrastructure · Jun 2024 – Mar 2025 · Bern + Zurich]
  #v(cv-subheading-after)
  #cv-b[Presented eight-figure infrastructure investments in SteerCo; discussed options with senior stakeholders]
  #v(cv-bullet-after)
  #cv-b[Supported CHF 10m+ supplier negotiations; identified CHF 100k+ immediate savings potential]
  #v(cv-bullet-after)
  #cv-b[Modelled cloud economics and 2× compute density under DC constraints; selected for the TOM workstream]
  #cv-entry-gap()
  // ccvl-station: airbus
  #cv-h[AI Engineering: #brand[AIRBUS Defence & Space]]
  #v(cv-heading-after)
  #cv-s[Risk & Compliance Analyst | AI/ML Master's Thesis · Jul 2023 – Mar 2024 · Ingolstadt]
  #v(cv-subheading-after)
  #cv-b[Analysed 20+ years of safety-critical data with ML; produced signals for risk and cost analyses]
  #v(cv-bullet-after)
  #cv-b[Single case: quantified six-figure annual savings potential; triggered eight-figure multi-site investment]
  #v(cv-bullet-after)
  #cv-b[Tailored 0-to-1 AI pilot to three departments’ objectives; secured stakeholder buy-in with business case]
  #cv-entry-gap()
  // ccvl-station: covendit
  #cv-h[M&A & Corporate Finance: #brand[COVENDIT]]
  #v(cv-heading-after)
  #cv-s[Investment Banking Analyst | Working Student · Apr 2022 – Jun 2022 · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[Supported live buy- and sell-side M&A mandates; built DCF/multiples Excel models, teasers and IMs]
  #v(cv-bullet-after)
  #cv-b[Built AI-assisted longlisting before ChatGPT; automated target screening and cut research time by 80%]
  #v(cv-bullet-after)
  #cv-b[Advised PE clients on targets; won a retainer and received an Associate-level return offer]
  #cv-entry-gap()
  // ccvl-station: nexgen
  #cv-h[Strategy & Technology Consulting: #brand[NEXGEN Business Consultants]]
  #v(cv-heading-after)
  #cv-s[Junior Consultant (Working Student) | Banking Technology & Regulation · Apr 2022 – Jun 2022 · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[BAIT | MaRisk: Translated rules for T+1 settlement into Tier-1 banking IT cloud migration guidance]
  #v(cv-bullet-after)
  #cv-b[Diagnosed an ETL bottleneck for a client pitch; cut processing time by 99%, from 24 hours to 15 minutes]
  #v(cv-bullet-after)
  #cv-b[Prepared regulatory/IT analysis for thought leadership and pitches; supported business development]
  #cv-entry-gap()
  // ccvl-station: consulting-venture
  #cv-h[Management & Technology Consulting: #brand[A Softer Space & Corbet Consulting]]
  #v(cv-heading-after)
  #cv-s[Head of Business Development | Management Consultant · Jan 2018 – Jun 2023 · CH, DE, IS, UK]
  #v(cv-subheading-after)
  #cv-b[Scaled trusted-advisor consulting sales to mid-six-figure revenue across four European markets]
  #v(cv-bullet-after)
  #cv-b[Delivered management | IT engagements across leadership | process | cloud | DLT from analysis to delivery]
  #v(cv-bullet-after)
  #cv-b[Managed project P&L end to end: acquisition, proposals, pricing, contracts, budgets, margins and cash flow]
  #cv-entry-gap()
  // ccvl-station: student-consulting
  #cv-h[Student Management & Innovation Consulting]
  #v(cv-heading-after)
  #cv-s[GREEN Finance Consulting (BDSU) | Enactus | AIESEC · 2016 – 2023 · 2 semesters each · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[GREEN: Scaled scholarship operations to 10× capacity; developed database system for Roland Berger]
  #v(cv-bullet-after)
  #cv-b[ENACTUS X: Co-built social venture for homeless people; created jobs and generated media coverage]
  #v(cv-bullet-after)
  #cv-b[AIESEC: Coordinated international placements with DAX firms; digitised talent-team workflows via CRM]
  #cv-entry-gap()
  // ccvl-station: teaching-research-venture
  #cv-h[Teaching, Market Research & Entrepreneurship]
  #v(cv-heading-after)
  #cv-s[Goethe University Frankfurt | multiple employers | self-employed · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[Elected tutor for three consecutive years & private tutor: applied statistics (SPSS, Python, R) and maths]
  #v(cv-bullet-after)
  #cv-b[Market research: interviewed 50+ CEOs and analysed 1,000+ calls; produced analyses and dashboards]
  #v(cv-bullet-after)
  #cv-b[Built and ran my own side venture for 16 years, spanning technical services, repairs and eCommerce]
]

#cv-pagebreak()

#block(breakable: false)[
  #cv-compact-heading[Education]
  #cv-hu[Scholarships: *Studienstiftung (Top 1%) | CDI (Top 4%, fully funded) | Sandvoss (MSc & BSc)*]
  #cv-entry-gap()
  // ccvl-station: executive-education
  #cv-h[Executive Education]
  #v(cv-heading-after)
  #cv-s[Collège des Ingénieurs (CDI) · Paris – Munich – Turin · 2024 – 2025 · Average grade: A (GPA 4.0)]
  #v(cv-subheading-after)
  #cv-b[Summer School: Advised #brand[Schwarz Digits] as Junior Consultant on the EU AI Act; assessed its implications]
  #v(cv-bullet-after)
  #cv-b[Case studies: Project finance (NPV/ROI), scenario analysis and capital allocation under uncertainty]
  #cv-entry-gap()
  // ccvl-station: physics-degrees
  #cv-h[M.Sc. & B.Sc. Physics]
  #v(cv-heading-after)
  #cv-s[Goethe University Frankfurt · Graduated 2024 · Grade: 1.0 (DE) | 6.0 (CH) | GPA 4.0]
  #v(cv-subheading-after)
  #cv-b[Focus: AI/ML (1.0) | high-tech IP (1.15) | electronics (1.3) | biophysics (1.3) | chemistry (1.0)]
  #v(cv-bullet-after)
  #cv-b[Research: Near-infrared spectroscopy | terahertz imaging | accelerator physics (LINAC)]
  #cv-entry-gap()
  // ccvl-station: psychology-degree
  #cv-h[B.Sc. Psychology]
  #v(cv-heading-after)
  #cv-s[Goethe University Frankfurt · Graduated 2017 · Grade: 1.6 (DE) | 5.6 (CH) | GPA 3.7]
  #v(cv-subheading-after)
  #cv-b[Focus: AI/ML & neuroscience | AR/VR training | clinical/organisational psychology (1.0)]
  #v(cv-bullet-after)
  #cv-b[Research at #brand[FIAS] (9 mos.): Modelled stereovision and neural tuning with ML; quantified empathy]
  #cv-entry-gap()
  #cv-hu[Matura (Abitur): *1.0 (DE) | 6.0 (CH) · Top of graduating class · Maths Olympiad · Student Academy*]
]

#block(breakable: false)[
  #cv-compact-heading[Professional Development]
  // ccvl-station: certificates
  #cv-h[Certifications & Training]
  #v(cv-heading-after)
  #cv-s[Finance | Data Analytics | GenAI | Leadership]
  #v(cv-subheading-after)
  #cv-b[CFI certification programmes (ongoing): BIDA | CBCA | CMSA | FMVA; training: Excel (VBA) | BI (Tableau)]
  #v(cv-bullet-after)
  #cv-b[Additional training: GenAI | automation | public speaking | negotiation | leadership | communication]
  #cv-entry-gap()
  // ccvl-station: consulting-finance-networks
  #cv-h[Consulting & Finance Networks]
  #v(cv-heading-after)
  #cv-s[Market proximity through regular exchange with practitioners and experienced sparring partners]
  #v(cv-subheading-after)
  #cv-b[At university: Bain Spark | BCG Emeralds | WFI Consulting Cup; BDSU & Studienstiftung alumnus]
  #v(cv-bullet-after)
  #cv-b[SECA Young Member; connected with practitioners across Swiss PE, VC and Corporate Development]
  #cv-entry-gap()
  // ccvl-station: technology-communities
  #cv-h[Tech Communities & Conferences]
  #v(cv-heading-after)
  #cv-s[Close to emerging technologies, tools and practical applications]
  #v(cv-subheading-after)
  #cv-b[Co-organised Swiss Python Summit & Web Zurich (2025); AV operations and speaker coordination]
  #v(cv-bullet-after)
  #cv-b[Digitale Gesellschaft | LUG | digitalswitzerland | Impact Hub; focus: GenAI & digital sovereignty]
]

#block(breakable: false)[
  #cv-compact-heading[Engagement]
  // ccvl-station: crisis-support
  #cv-h[Harm Reduction & Crisis Support]
  #v(cv-heading-after)
  #cv-s(min-fill: 25, target-fill: 35)[First aid | psychosocial de-escalation]
  #v(cv-subheading-after)
  #cv-b[Intervened in life-threatening situations multiple times; provided first aid and ensured EMS handover]
  #v(cv-bullet-after)
  #cv-b[De-escalated acute psychosocial crises; stabilised, oriented and referred people to specialist support]
  #cv-entry-gap()
  // ccvl-station: mentoring
  #cv-h[Counselling, Mentoring & Student Representation]
  #v(cv-heading-after)
  #cv-s[Online youth counselling | cross-disciplinary knowledge transfer]
  #v(cv-subheading-after)
  #cv-b[One of few male Kids Hotline counsellors; supported youth on identity, body image & self-doubt]
  #v(cv-bullet-after)
  #cv-b[Co-developed psychology mentoring across cohorts; supported Physics Student Council & Night of Science]
]

#block(breakable: false)[
  #cv-compact-heading[Personal]
  // ccvl-station: family-responsibility
  #cv-h[Educational Mobility & Family Responsibility]
  #v(cv-heading-after)
  #cv-s[First-generation academic | education | entrepreneurship | care coordination]
  #v(cv-subheading-after)
  #cv-b[Supported siblings personally & financially: top-grade Abitur (1.0) | medical studies | company launch]
  #v(cv-bullet-after)
  #cv-b[Took family care leave in 2025; coordinated care, financing & long-term support for my mother]
  #cv-entry-gap()
  // ccvl-station: open-source-community
  #cv-h[Intercultural Community & Open-Source Software]
  #v(cv-heading-after)
  #cv-s[Shared living | international FOSS collaboration]
  #v(cv-subheading-after)
  #cv-b[Lived with 20+ people from 10+ countries; actively fostered intercultural exchange through shared living]
  #v(cv-bullet-after)
  #cv-b[Published 50+ open-source projects; contributed to other projects, most recently oo7 (cybersecurity)]
]

#if cv-pages >= 3 [
  #cv-pagebreak()

  #cv-superheading[Projects & Initiatives]
  #block(breakable: false)[
    #cv-spacious-heading[Ongoing]
    // ccvl-project: product-innovation
    #cv-h[Product Innovation & Engineering]
    #v(cv-heading-after)
    #cv-s[Product releases 2026: local-first AI | remote development | systems UX]
    #v(cv-subheading-after)
    #cv-b[cfetch: local-first AI memory (RAG) | up to 93.4% less wasted context | \>15% token-saving potential]
    #v(cv-bullet-after)
    #cv-b[cterm: remote-first coding terminal | dotkeeper: P2P code sync | cbar: cross-machine 2D app launcher]
    #cv-entry-gap()
    // ccvl-project: declarative-systems-platform
    #cv-h[Declarative Systems Platform]
    #v(cv-heading-after)
    #cv-s[Product releases 2026: 50+ reusable components for NixOS, Arch & GCP]
    #v(cv-subheading-after)
    #cv-b[Unified hosts, storage, networks, desktops & apps across NixOS and Arch in one reproducible platform]
    #v(cv-bullet-after)
    #cv-b[Deployed NixOS, k3s, Argo CD & OpenTofu on GCP & bare metal | signed updates | health checks | rollback]
    #cv-entry-gap()
    // ccvl-project: content-innovation
    #cv-h[Content Innovation & AI-Enabled Media]
    #v(cv-heading-after)
    #cv-s[New formats: 4K multi-camera video, GenAI & resilient audio · since 2025]
    #v(cv-subheading-after)
    #cv-b[Produced 4K multi-camera video for Swiss Python Summit, Winter Congress & CoSin (Chaos Singularity)]
    #v(cv-bullet-after)
    #cv-b[Ran GPU-hosted ComfyUI for GenAI media | caudio: audio routing across 3 hosts with failover & recovery]
    #cv-entry-gap()
    // ccvl-project: careervector-jobcache
    #cv-h[CareerVector & JobCache]
    #v(cv-heading-after)
    #cv-s[AI-native career platform & job-data pipeline · live since 2025]
    #v(cv-subheading-after)
    #cv-b[CareerVector: AI-native career platform across collaborative web, desktop & terminal workflows with Typst]
    #v(cv-bullet-after)
    #cv-b[JobCache: built 91 Rust adapters for continuous, distributed ingestion & deduplication of CH/EU job ads]
    #cv-entry-gap()
    // ccvl-project: private-ai-cloud
    #cv-h[Private AI & Cloud Platform]
    #v(cv-heading-after)
    #cv-s[Digitally sovereign production platform for 10+ users · since 2024]
    #v(cv-subheading-after)
    #cv-b[Operated 30+ private services and 100+ TB with SSO, monitoring, automated backups & disaster recovery]
    #v(cv-bullet-after)
    #cv-b[Ran local LLMs & AI agents in production on shared GPU infrastructure | \>90% lower cost than public cloud]
  ]

  #block(breakable: false)[
    #cv-spacious-heading[Delivered]
    // ccvl-project: management-buy-in
    #cv-h[Management Buy-In: Deal Origination & Due Diligence]
    #v(cv-heading-after)
    #cv-s[Indian IT outsourcer · independent MBI through the purchase decision · 2022]
    #v(cv-subheading-after)
    #cv-b[Identified an Indian IT outsourcing target for an MBI and independently conducted end-to-end due diligence]
    #v(cv-bullet-after)
    #cv-b[Built the acquisition thesis; assessed strategic fit, opportunities & risks through the final go/no-go decision]
    #cv-entry-gap()
    // ccvl-project: solar-recovery
    #cv-h[Solar SME: Incident Recovery & Cloud Migration]
    #v(cv-heading-after)
    #cv-s[Business-critical systems for sales & field service · 2022]
    #v(cv-subheading-after)
    #cv-b[Restored the core system on day one and kept sales & field service operational until full replacement]
    #v(cv-bullet-after)
    #cv-b[Tested cloud migration options against operating needs; prevented six-to-seven-figure misinvestment]
    #cv-entry-gap()
    // ccvl-project: leadership-digital-pivot
    #cv-h[Leadership Advisory: Digital Pivot]
    #v(cv-heading-after)
    #cv-s[Frankfurt-based leadership brand · hybrid delivery & new sales channels · 2022]
    #v(cv-subheading-after)
    #cv-b[Redesigned Performance Leadership offering for scalable hybrid delivery and built on-demand infrastructure]
    #v(cv-bullet-after)
    #cv-b[Aligned funnel to customer pain points; diversified revenue and placed courses with Haufe Akademie]
    #cv-entry-gap()
    // ccvl-project: crypto-infrastructure
    #cv-h[Crypto Infrastructure: Business Case & Operations]
    #v(cv-heading-after)
    #cv-s[Mining pilots from business case to stable operations · multiple clients · 2021]
    #v(cv-subheading-after)
    #cv-b[Sized and costed pilot and operating model; sourced hardware and actively managed operating risks]
    #v(cv-bullet-after)
    #cv-b[Delivered monitored, stable mining operation on schedule; optimised hash rate via custom firmware]
    #cv-entry-gap()
    // ccvl-project: it-services-ecommerce
    #cv-h[IT Services & Automated eCommerce]
    #v(cv-heading-after)
    #cv-s[Independent business · exited at 80% of book value · 2009 – 2025]
    #v(cv-subheading-after)
    #cv-b[Built and ran a hardware business for SME & B2C clients: sales | custom builds | diagnostics | repairs]
    #v(cv-bullet-after)
    #cv-b[Automated listings, inventory, tracking and logistics for five-figure eBay operation through own mini-ERP]
  ]
]

// Page 4 is a machine-retrieval layer: its noun-based entries may include
// adjacent and independently developed knowledge, but never imply employment,
// ownership or results. Use literal ASCII pipes with spaces between list items,
// keep canonical phrases intact, and target 92-98% width without wrapping.
// Layout contract: 3 pillars x 3 subheadings x 3 rows. Never rebalance the counts.
#if cv-pages >= 4 [
  #cv-pagebreak()

  #cv-superheading[Capabilities & AI Keywords]
  #block(breakable: false)[
    #cv-spacious-heading[AI, Software & Data]
    // ccvl-competency: ai-products-tooling
    #cv-h[AI Products, Tooling & Model Ecosystems]
    #v(cv-competency-heading-after)
    #cv-b[US AI Tooling: Anthropic Claude Code | Claude Desktop | OpenAI Codex | Codex App | ChatGPT Desktop]
    #v(cv-bullet-after)
    #cv-b[Open Source AI Tooling: OpenCode | Ollama | vLLM | llama.cpp | Open WebUI | Hugging Face | Langfuse]
    #v(cv-bullet-after)
    #cv-b[Models: GPT | Claude | Gemini | GLM | DeepSeek | Qwen | Kimi | MiniMax | MiMo | Llama | Mistral | Gemma]
    #cv-entry-gap()
    // ccvl-competency: applied-ai-data
    #cv-h[AI Engineering, Agents & Data Science]
    #v(cv-competency-heading-after)
    #cv-b[LLM Engineering: Large Language Models (LLMs) | RAG | Embeddings | Vector Search | Evaluations]
    #v(cv-bullet-after)
    #cv-b[Agentic Systems: AI Agents | Multi-Agent Systems | Model Context Protocol (MCP) | Agent SDKs | Tool Use]
    #v(cv-bullet-after)
    #cv-b[Data Science & ML: Statistics | Machine Learning | Time Series | Predictive Modelling | Experiments | R]
    #cv-entry-gap()
    // ccvl-competency: software-infrastructure
    #cv-h[Software Engineering, Web & Platforms]
    #v(cv-competency-heading-after)
    #cv-b[Engineering: Python | Rust | Go | Java | TypeScript | JavaScript | Bash | SQL | Git | CI/CD | Testing]
    #v(cv-bullet-after)
    #cv-b[Web & Publishing: Svelte | Astro | HTML | CSS | REST | GraphQL | WebSockets | Markdown | Typst]
    #v(cv-bullet-after)
    #cv-b[Cloud & Data Platforms: PostgreSQL | Data Pipelines | Linux | Nix/NixOS | Kubernetes | GitOps | OpenTofu]
    #cv-spacious-heading[Strategy, Innovation & Transformation]
    // ccvl-competency: innovation-management
    #cv-h[Innovation Management & Emerging Technologies]
    #v(cv-competency-heading-after)
    #cv-b[Innovation Management: Innovation Pipeline | Stage-Gate | Incremental Innovation | Disruptive Innovation]
    #v(cv-bullet-after)
    #cv-b[Technology Scouting: Emerging Technologies | Trend Analysis | Horizon Scanning | Technology Assessment]
    #v(cv-bullet-after)
    #cv-b[Product Innovation: Product Discovery | Prototyping | Proof of Concept (PoC) | MVP | Market Validation]
    #cv-entry-gap()
    // ccvl-competency: strategy
    #cv-h[Corporate, Growth & Technology Strategy]
    #v(cv-competency-heading-after)
    #cv-b[Corporate Strategy: Strategic Planning | Scenario Planning | Competitive Analysis | Decision Support]
    #v(cv-bullet-after)
    #cv-b[Growth Strategy: Business Development | Market Entry | Go-to-Market | Partnerships | Pricing | B2B]
    #v(cv-bullet-after)
    #cv-b[Technology Strategy: AI Strategy | Roadmaps | Business Cases | Enterprise Architecture | TCO | FinOps]
    #cv-entry-gap()
    // ccvl-competency: transformation-governance
    #cv-h[Transformation, Operating Models & Governance]
    #v(cv-competency-heading-after)
    #cv-b[Operating Models: Target Operating Model (TOM) | Organisational Design | Decision Rights | Role Design]
    #v(cv-bullet-after)
    #cv-b[Change & Value Creation: AI Adoption | Change Management | Benefits Realisation | Cost Transformation]
    #v(cv-bullet-after)
    #cv-b[AI Governance: EU AI Act | Responsible AI | Model Risk | DORA | Operational Resilience | GDPR]
    #cv-spacious-heading[Finance, Investments & Markets]
    // ccvl-competency: finance-ma
    #cv-h[Finance Transformation, Corporate Finance & M&A]
    #v(cv-competency-heading-after)
    #cv-b[CFO Agenda: Finance Platform | Finance Data Architecture | Planning & Forecasting | AI-enabled Finance]
    #v(cv-bullet-after)
    #cv-b[Corporate Finance: Financial Modelling | Valuation | DCF | Multiples | Project Finance | NPV | IRR | DSCR]
    #v(cv-bullet-after)
    #cv-b[M&A: Target Screening | Financial Due Diligence | Synergy Assessment | Post-Merger Integration (PMI)]
    #cv-entry-gap()
    // ccvl-competency: private-markets
    #cv-h[Private Markets & Investment Management]
    #v(cv-competency-heading-after)
    #cv-b[Private Markets: Private Equity | Private Credit | Infrastructure Investments | Real Estate | Secondaries]
    #v(cv-bullet-after)
    #cv-b[Investment Strategies: Buyouts | Growth Equity | Direct Lending | Distressed Debt | Special Situations]
    #v(cv-bullet-after)
    #cv-b[CIO Office: Investment Strategy | Multi-Asset | Portfolio Construction | Strategic Asset Allocation (SAA)]
    #cv-entry-gap()
    // ccvl-competency: trading-risk
    #cv-h[Trading, Quantitative Finance & Risk]
    #v(cv-competency-heading-after)
    #cv-b[Energy & Commodity Markets: Power Trading | Day-Ahead | Intraday | Gas/LNG | Metals | Carbon | Freight]
    #v(cv-bullet-after)
    #cv-b[Systematic Trading: Alpha Signals | Backtesting | Trade Execution | Futures | Swaps | Options | Hedging]
    #v(cv-bullet-after)
    #cv-b[Quantitative Risk: PnL | Value at Risk (VaR) | Stress Testing | Monte Carlo | Option Pricing | Volatility]
  ]
]

#assert-page-count(cv-pages)
