// Approved personal showcase entries, shared by this locale’s CV substyles.
// Never reuse these claims or wording for another person.
#let render-entries(cv) = {
  let (
    brand,
    cv-b,
    cv-bullet-after,
    cv-compact-heading,
    cv-competency-heading-after,
    cv-entry-gap,
    cv-gap,
    cv-h,
    cv-heading-after,
    cv-hu,
    cv-pagebreak,
    cv-pages,
    cv-s,
    cv-spacious-heading,
    cv-strings,
    cv-subheading-after,
    cv-superheading,
  ) = cv
  [
    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.experience]
      // ccvl-station: cenvion
      #cv-h[Infrastructure Investments & Asset Management: #brand[CENVION]]
      #cv-gap("heading_after_pt")
      #cv-s[Associate Intern | Infrastructure Investments · Jan 2026 – Mar 2026 (plus freelance work) · Wollerau (CH)]
      #cv-gap("subheading_after_pt")
      #cv-b[Introduced #brand[Claude] for investment reporting; embedded GenAI in the team's analysis and reporting workflows]
      #cv-gap("bullet_after_pt")
      #cv-b[Developed RAG-based AI search across project and portfolio data; made internal knowledge searchable]
      #cv-gap("bullet_after_pt")
      #cv-b[Built Excel project-finance models; analysed cash flows, returns and financing scenarios]
      #cv-entry-gap()
      // ccvl-station: swisscom
      #cv-h[Cloud Strategy & Transformation: #brand[Swisscom Financial Services]]
      #cv-gap("heading_after_pt")
      #cv-s[Executive Assistant & Consultant | B2B & Infrastructure · Jun 2024 – Mar 2025 · Bern + Zurich]
      #cv-gap("subheading_after_pt")
      #cv-b[Presented eight-figure infrastructure investments in SteerCo; discussed options with senior stakeholders]
      #cv-gap("bullet_after_pt")
      #cv-b[Supported CHF 10m+ supplier negotiations; identified CHF 100k+ immediate savings potential]
      #cv-gap("bullet_after_pt")
      #cv-b[Modelled cloud economics and 2× compute density under DC constraints; selected for the TOM workstream]
      #cv-entry-gap()
      // ccvl-station: airbus
      #cv-h[AI Engineering: #brand[AIRBUS Defence & Space]]
      #cv-gap("heading_after_pt")
      #cv-s[Risk & Compliance Analyst | AI/ML Master's Thesis · Jul 2023 – Mar 2024 · Ingolstadt]
      #cv-gap("subheading_after_pt")
      #cv-b[Analysed 20+ years of safety-critical data with ML; produced signals for risk and cost analyses]
      #cv-gap("bullet_after_pt")
      #cv-b[Single case: quantified six-figure annual savings potential; triggered eight-figure multi-site investment]
      #cv-gap("bullet_after_pt")
      #cv-b[Tailored 0-to-1 AI pilot to three departments’ objectives; secured stakeholder buy-in with business case]
      #cv-entry-gap()
      // ccvl-station: covendit
      #cv-h[M&A & Corporate Finance: #brand[COVENDIT]]
      #cv-gap("heading_after_pt")
      #cv-s[Investment Banking Analyst | Working Student · Apr 2022 – Jun 2022 · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[Supported live buy- and sell-side M&A mandates; built DCF/multiples Excel models, teasers and IMs]
      #cv-gap("bullet_after_pt")
      #cv-b[Built AI-assisted longlisting before ChatGPT; automated target screening and cut research time by 80%]
      #cv-gap("bullet_after_pt")
      #cv-b[Advised PE clients on targets; won a retainer and received an Associate-level return offer]
      #cv-entry-gap()
      // ccvl-station: nexgen
      #cv-h[Strategy & Technology Consulting: #brand[NEXGEN Business Consultants]]
      #cv-gap("heading_after_pt")
      #cv-s[Junior Consultant (Working Student) | Banking Technology & Regulation · Apr 2022 – Jun 2022 · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[BAIT | MaRisk: Translated rules for T+1 settlement into Tier-1 banking IT cloud migration guidance]
      #cv-gap("bullet_after_pt")
      #cv-b[Diagnosed an ETL bottleneck for a client pitch; cut processing time by 99%, from 24 hours to 15 minutes]
      #cv-gap("bullet_after_pt")
      #cv-b[Prepared regulatory/IT analysis for thought leadership and pitches; supported business development]
      #cv-entry-gap()
      // ccvl-station: consulting-venture
      #cv-h[Management & Technology Consulting: #brand[A Softer Space & Corbet Consulting]]
      #cv-gap("heading_after_pt")
      #cv-s[Head of Business Development | Management Consultant · Jan 2018 – Jun 2023 · CH, DE, IS, UK]
      #cv-gap("subheading_after_pt")
      #cv-b[Scaled trusted-advisor consulting sales to mid-six-figure revenue across four European markets]
      #cv-gap("bullet_after_pt")
      #cv-b[Delivered management | IT engagements across leadership | process | cloud | DLT from analysis to delivery]
      #cv-gap("bullet_after_pt")
      #cv-b[Managed project P&L end to end: acquisition, proposals, pricing, contracts, budgets, margins and cash flow]
      #cv-entry-gap()
      // ccvl-station: student-consulting
      #cv-h[Student Management & Innovation Consulting]
      #cv-gap("heading_after_pt")
      #cv-s[GREEN Finance Consulting (BDSU) | Enactus | AIESEC · 2016 – 2023 · 2 semesters each · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[GREEN: Scaled scholarship operations to 10× capacity; developed database system for Roland Berger]
      #cv-gap("bullet_after_pt")
      #cv-b[ENACTUS X: Co-built social venture for homeless people; created jobs and generated media coverage]
      #cv-gap("bullet_after_pt")
      #cv-b[AIESEC: Coordinated international placements with DAX firms; digitised talent-team workflows via CRM]
      #cv-entry-gap()
      // ccvl-station: teaching-research-venture
      #cv-h[Teaching, Market Research & Entrepreneurship]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe University Frankfurt | multiple employers | self-employed · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[Elected tutor for three consecutive years & private tutor: applied statistics (SPSS, Python, R) and maths]
      #cv-gap("bullet_after_pt")
      #cv-b[Market research: interviewed 50+ CEOs and analysed 1'000+ calls; produced analyses and dashboards]
      #cv-gap("bullet_after_pt")
      #cv-b[Built and ran my own side venture for 16 years, spanning technical services, repairs and eCommerce]
    ]

    #cv-pagebreak()

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.education]
      #cv-hu[Scholarships: *Studienstiftung (Top 1%) | CDI (Top 4%, fully funded) | Sandvoss (MSc & BSc)*]
      #cv-entry-gap()
      // ccvl-station: executive-education
      #cv-h[Executive Education]
      #cv-gap("heading_after_pt")
      #cv-s[Collège des Ingénieurs (CDI) · Paris – Munich – Turin · 2024 – 2025 · Average grade: A (GPA 4.0)]
      #cv-gap("subheading_after_pt")
      #cv-b[Summer School: Advised #brand[Schwarz Digits] as Junior Consultant on the EU AI Act; assessed its implications]
      #cv-gap("bullet_after_pt")
      #cv-b[Case studies: Project finance (NPV/ROI), scenario analysis and capital allocation under uncertainty]
      #cv-entry-gap()
      // ccvl-station: physics-degrees
      #cv-h[M.Sc. & B.Sc. Physics]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe University Frankfurt · Graduated 2024 · Grade: 1.0 (DE) | 6.0 (CH) | GPA 4.0]
      #cv-gap("subheading_after_pt")
      #cv-b[Focus: AI/ML (1.0) | high-tech IP (1.15) | electronics (1.3) | biophysics (1.3) | chemistry (1.0)]
      #cv-gap("bullet_after_pt")
      #cv-b[Research: Near-infrared spectroscopy | terahertz imaging | accelerator physics (LINAC)]
      #cv-entry-gap()
      // ccvl-station: psychology-degree
      #cv-h[B.Sc. Psychology]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe University Frankfurt · Graduated 2017 · Grade: 1.6 (DE) | 5.6 (CH) | GPA 3.7]
      #cv-gap("subheading_after_pt")
      #cv-b[Focus: AI/ML & neuroscience | AR/VR training | clinical/organisational psychology (1.0)]
      #cv-gap("bullet_after_pt")
      #cv-b[Research at #brand[FIAS] (9 mos.): Modelled stereovision and neural tuning with ML; quantified empathy]
      #cv-entry-gap()
      #cv-hu[Matura (Abitur): *1.0 (DE) | 6.0 (CH) · Top of graduating class · Maths Olympiad · Student Academy*]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.professional_development]
      // ccvl-station: certificates
      #cv-h[Certifications & Training]
      #cv-gap("heading_after_pt")
      #cv-s[Finance | Data Analytics | GenAI | Leadership]
      #cv-gap("subheading_after_pt")
      #cv-b[CFI certification programmes (ongoing): BIDA | CBCA | CMSA | FMVA; training: Excel (VBA) | BI (Tableau)]
      #cv-gap("bullet_after_pt")
      #cv-b[Additional training: GenAI | automation | public speaking | negotiation | leadership | communication]
      #cv-entry-gap()
      // ccvl-station: consulting-finance-networks
      #cv-h[Consulting & Finance Networks]
      #cv-gap("heading_after_pt")
      #cv-s[Market proximity through regular exchange with practitioners and experienced sparring partners]
      #cv-gap("subheading_after_pt")
      #cv-b[At university: Bain Spark | BCG Emeralds | WFI Consulting Cup; BDSU & Studienstiftung alumnus]
      #cv-gap("bullet_after_pt")
      #cv-b[SECA Young Member; connected with practitioners across Swiss PE, VC and Corporate Development]
      #cv-entry-gap()
      // ccvl-station: technology-communities
      #cv-h[Tech Communities & Conferences]
      #cv-gap("heading_after_pt")
      #cv-s[Close to emerging technologies, tools and practical applications]
      #cv-gap("subheading_after_pt")
      #cv-b[Co-organised Swiss Python Summit & Web Zurich (2025); AV operations and speaker coordination]
      #cv-gap("bullet_after_pt")
      #cv-b[Digitale Gesellschaft | LUG | digitalswitzerland | Impact Hub; focus: GenAI & digital sovereignty]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.engagement]
      // ccvl-station: crisis-support
      #cv-h[Harm Reduction & Crisis Support]
      #cv-gap("heading_after_pt")
      #cv-s(min-fill: 25, target-fill: 35)[First aid | psychosocial de-escalation]
      #cv-gap("subheading_after_pt")
      #cv-b[Intervened in life-threatening situations multiple times; provided first aid and ensured EMS handover]
      #cv-gap("bullet_after_pt")
      #cv-b[De-escalated acute psychosocial crises; stabilised, oriented and referred people to specialist support]
      #cv-entry-gap()
      // ccvl-station: mentoring
      #cv-h[Counselling, Mentoring & Student Representation]
      #cv-gap("heading_after_pt")
      #cv-s[Online youth counselling | cross-disciplinary knowledge transfer]
      #cv-gap("subheading_after_pt")
      #cv-b[One of few male Kids Hotline counsellors; supported youth on identity, body image & self-doubt]
      #cv-gap("bullet_after_pt")
      #cv-b[Co-developed psychology mentoring across cohorts; supported Physics Student Council & Night of Science]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.personal]
      // ccvl-station: family-responsibility
      #cv-h[Educational Mobility & Family Responsibility]
      #cv-gap("heading_after_pt")
      #cv-s[First-generation academic | education | entrepreneurship | care coordination]
      #cv-gap("subheading_after_pt")
      #cv-b[Supported siblings personally & financially: top-grade Abitur (1.0) | medical studies | company launch]
      #cv-gap("bullet_after_pt")
      #cv-b[Took family care leave in 2025; coordinated care, financing & long-term support for my mother]
      #cv-entry-gap()
      // ccvl-station: open-source-community
      #cv-h[Intercultural Community & Open-Source Software]
      #cv-gap("heading_after_pt")
      #cv-s[Shared living | international FOSS collaboration]
      #cv-gap("subheading_after_pt")
      #cv-b[Lived with 20+ people from 10+ countries; actively fostered intercultural exchange through shared living]
      #cv-gap("bullet_after_pt")
      #cv-b[Published 50+ open-source projects; contributed to other projects, most recently oo7 (cybersecurity)]
    ]

    #if cv-pages >= 3 [
      #cv-pagebreak()

      #cv-superheading[#cv-strings.projects]
      #block(breakable: false)[
        #cv-spacious-heading[#cv-strings.projects_ongoing]
        // ccvl-project: product-innovation
        #cv-h[Product Innovation & Engineering]
        #cv-gap("heading_after_pt")
        #cv-s[Product releases 2026: local-first AI | remote development | systems UX]
        #cv-gap("subheading_after_pt")
        #cv-b[cfetch: local-first AI memory (RAG) | up to 93.4% less wasted context | \>15% token-saving potential]
        #cv-gap("bullet_after_pt")
        #cv-b[cterm: remote-first coding terminal | dotkeeper: P2P code sync | cbar: cross-machine 2D app launcher]
        #cv-entry-gap()
        // ccvl-project: declarative-systems-platform
        #cv-h[Declarative Systems Platform]
        #cv-gap("heading_after_pt")
        #cv-s[Product releases 2026: 50+ reusable components for NixOS, Arch & GCP]
        #cv-gap("subheading_after_pt")
        #cv-b[Unified hosts, storage, networks, desktops & apps across NixOS and Arch in one reproducible platform]
        #cv-gap("bullet_after_pt")
        #cv-b[Deployed NixOS, k3s, Argo CD & OpenTofu on GCP & bare metal | signed updates | health checks | rollback]
        #cv-entry-gap()
        // ccvl-project: content-innovation
        #cv-h[Content Innovation & AI-Enabled Media]
        #cv-gap("heading_after_pt")
        #cv-s[New formats: 4K multi-camera video, GenAI & resilient audio · since 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[Produced 4K multi-camera video for Swiss Python Summit, Winter Congress & CoSin (Chaos Singularity)]
        #cv-gap("bullet_after_pt")
        #cv-b[Ran GPU-hosted ComfyUI for GenAI media | caudio: audio routing across 3 hosts with failover & recovery]
        #cv-entry-gap()
        // ccvl-project: careervector-jobcache
        #cv-h[CareerVector & JobCache]
        #cv-gap("heading_after_pt")
        #cv-s[AI-native career platform & job-data pipeline · live since 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[CareerVector: AI-native career platform across collaborative web, desktop & terminal workflows with Typst]
        #cv-gap("bullet_after_pt")
        #cv-b[JobCache: built 91 Rust adapters for continuous, distributed ingestion & deduplication of CH/EU job ads]
        #cv-entry-gap()
        // ccvl-project: private-ai-cloud
        #cv-h[Private AI & Cloud Platform]
        #cv-gap("heading_after_pt")
        #cv-s[Digitally sovereign production platform for 10+ users · since 2024]
        #cv-gap("subheading_after_pt")
        #cv-b[Operated 30+ private services and 100+ TB with SSO, monitoring, automated backups & disaster recovery]
        #cv-gap("bullet_after_pt")
        #cv-b[Ran local LLMs & AI agents in production on shared GPU infrastructure | \>90% lower cost than public cloud]
      ]

      #block(breakable: false)[
        #cv-spacious-heading[#cv-strings.projects_delivered]
        // ccvl-project: management-buy-in
        #cv-h[Management Buy-In: Deal Origination & Due Diligence]
        #cv-gap("heading_after_pt")
        #cv-s[Indian IT outsourcer · independent MBI through the purchase decision · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Identified an Indian IT outsourcing target for an MBI and independently conducted end-to-end due diligence]
        #cv-gap("bullet_after_pt")
        #cv-b[Built the acquisition thesis; assessed strategic fit, opportunities & risks through the final go/no-go decision]
        #cv-entry-gap()
        // ccvl-project: solar-recovery
        #cv-h[Solar SME: Incident Recovery & Cloud Migration]
        #cv-gap("heading_after_pt")
        #cv-s[Business-critical systems for sales & field service · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Restored the core system on day one and kept sales & field service operational until full replacement]
        #cv-gap("bullet_after_pt")
        #cv-b[Tested cloud migration options against operating needs; prevented six-to-seven-figure misinvestment]
        #cv-entry-gap()
        // ccvl-project: leadership-digital-pivot
        #cv-h[Leadership Advisory: Digital Pivot]
        #cv-gap("heading_after_pt")
        #cv-s[Frankfurt-based leadership brand · hybrid delivery & new sales channels · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Redesigned Performance Leadership offering for scalable hybrid delivery and built on-demand infrastructure]
        #cv-gap("bullet_after_pt")
        #cv-b[Aligned funnel to customer pain points; diversified revenue and placed courses with Haufe Akademie]
        #cv-entry-gap()
        // ccvl-project: crypto-infrastructure
        #cv-h[Crypto Infrastructure: Business Case & Operations]
        #cv-gap("heading_after_pt")
        #cv-s[Mining pilots from business case to stable operations · multiple clients · 2021]
        #cv-gap("subheading_after_pt")
        #cv-b[Sized and costed pilot and operating model; sourced hardware and actively managed operating risks]
        #cv-gap("bullet_after_pt")
        #cv-b[Delivered monitored, stable mining operation on schedule; optimised hash rate via custom firmware]
        #cv-entry-gap()
        // ccvl-project: it-services-ecommerce
        #cv-h[IT Services & Automated eCommerce]
        #cv-gap("heading_after_pt")
        #cv-s[Independent business · exited at 80% of book value · 2009 – 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[Built and ran a hardware business for SME & B2C clients: sales | custom builds | diagnostics | repairs]
        #cv-gap("bullet_after_pt")
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

      #cv-superheading[#cv-strings.capabilities]
      #block(breakable: false)[
        #cv-spacious-heading[#cv-strings.pillar_ai]
        // ccvl-competency: ai-products-tooling
        #cv-h[AI Products, Tooling & Model Ecosystems]
        #cv-gap("competency_heading_after_pt")
        #cv-b[US AI Tooling: Anthropic Claude Code | Claude Desktop | OpenAI Codex | Codex App | ChatGPT Desktop]
        #cv-gap("bullet_after_pt")
        #cv-b[Open Source AI Tooling: OpenCode | Ollama | vLLM | llama.cpp | Open WebUI | Hugging Face | Langfuse]
        #cv-gap("bullet_after_pt")
        #cv-b[Models: GPT | Claude | Gemini | GLM | DeepSeek | Qwen | Kimi | MiniMax | MiMo | Llama | Mistral | Gemma]
        #cv-entry-gap()
        // ccvl-competency: applied-ai-data
        #cv-h[AI Engineering, Agents & Data Science]
        #cv-gap("competency_heading_after_pt")
        #cv-b[LLM Engineering: Large Language Models (LLMs) | RAG | Embeddings | Vector Search | Evaluations]
        #cv-gap("bullet_after_pt")
        #cv-b[Agentic Systems: AI Agents | Multi-Agent Systems | Model Context Protocol (MCP) | Agent SDKs | Tool Use]
        #cv-gap("bullet_after_pt")
        #cv-b[Data Science & ML: Statistics | Machine Learning | Time Series | Predictive Modelling | Experiments | R]
        #cv-entry-gap()
        // ccvl-competency: software-infrastructure
        #cv-h[Software Engineering, Web & Platforms]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Engineering: Python | Rust | Go | Java | TypeScript | JavaScript | Bash | SQL | Git | CI/CD | Testing]
        #cv-gap("bullet_after_pt")
        #cv-b[Web & Publishing: Svelte | Astro | HTML | CSS | REST | GraphQL | WebSockets | Markdown | Typst]
        #cv-gap("bullet_after_pt")
        #cv-b[Cloud & Data Platforms: PostgreSQL | Data Pipelines | Linux | Nix/NixOS | Kubernetes | GitOps | OpenTofu]
        #cv-spacious-heading[#cv-strings.pillar_strategy]
        // ccvl-competency: innovation-management
        #cv-h[Innovation Management & Emerging Technologies]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Innovation Management: Innovation Pipeline | Stage-Gate | Incremental Innovation | Disruptive Innovation]
        #cv-gap("bullet_after_pt")
        #cv-b[Technology Scouting: Emerging Technologies | Trend Analysis | Horizon Scanning | Technology Assessment]
        #cv-gap("bullet_after_pt")
        #cv-b[Product Innovation: Product Discovery | Prototyping | Proof of Concept (PoC) | MVP | Market Validation]
        #cv-entry-gap()
        // ccvl-competency: strategy
        #cv-h[Corporate, Growth & Technology Strategy]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Corporate Strategy: Strategic Planning | Scenario Planning | Competitive Analysis | Decision Support]
        #cv-gap("bullet_after_pt")
        #cv-b[Growth Strategy: Business Development | Market Entry | Go-to-Market | Partnerships | Pricing | B2B]
        #cv-gap("bullet_after_pt")
        #cv-b[Technology Strategy: AI Strategy | Roadmaps | Business Cases | Enterprise Architecture | TCO | FinOps]
        #cv-entry-gap()
        // ccvl-competency: transformation-governance
        #cv-h[Transformation, Operating Models & Governance]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Operating Models: Target Operating Model (TOM) | Organisational Design | Decision Rights | Role Design]
        #cv-gap("bullet_after_pt")
        #cv-b[Change & Value Creation: AI Adoption | Change Management | Benefits Realisation | Cost Transformation]
        #cv-gap("bullet_after_pt")
        #cv-b[AI Governance: EU AI Act | Responsible AI | Model Risk | DORA | Operational Resilience | GDPR]
        #cv-spacious-heading[#cv-strings.pillar_finance]
        // ccvl-competency: finance-ma
        #cv-h[Finance Transformation, Corporate Finance & M&A]
        #cv-gap("competency_heading_after_pt")
        #cv-b[CFO Agenda: Finance Platform | Finance Data Architecture | Planning & Forecasting | AI-enabled Finance]
        #cv-gap("bullet_after_pt")
        #cv-b[Corporate Finance: Financial Modelling | Valuation | DCF | Multiples | Project Finance | NPV | IRR | DSCR]
        #cv-gap("bullet_after_pt")
        #cv-b[M&A: Target Screening | Financial Due Diligence | Synergy Assessment | Post-Merger Integration (PMI)]
        #cv-entry-gap()
        // ccvl-competency: private-markets
        #cv-h[Private Markets & Investment Management]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Private Markets: Private Equity | Private Credit | Infrastructure Investments | Real Estate | Secondaries]
        #cv-gap("bullet_after_pt")
        #cv-b[Investment Strategies: Buyouts | Growth Equity | Direct Lending | Distressed Debt | Special Situations]
        #cv-gap("bullet_after_pt")
        #cv-b[CIO Office: Investment Strategy | Multi-Asset | Portfolio Construction | Strategic Asset Allocation (SAA)]
        #cv-entry-gap()
        // ccvl-competency: trading-risk
        #cv-h[Trading, Quantitative Finance & Risk]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Energy & Commodity Markets: Power Trading | Day-Ahead | Intraday | Gas/LNG | Metals | Carbon | Freight]
        #cv-gap("bullet_after_pt")
        #cv-b[Systematic Trading: Alpha Signals | Backtesting | Trade Execution | Futures | Swaps | Options | Hedging]
        #cv-gap("bullet_after_pt")
        #cv-b[Quantitative Risk: PnL | Value at Risk (VaR) | Stress Testing | Monte Carlo | Option Pricing | Volatility]
      ]
    ]
  ]
}
