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
      #cv-s[Associate Intern | Infrastructure Investments · Jan 2026 – Mär 2026 (plus freie Mitarbeit) · Wollerau (CH)]
      #cv-gap("subheading_after_pt")
      #cv-b[#brand[Claude] für Investment Reporting eingeführt; GenAI in Analyse- und Berichtsworkflows des Teams integriert]
      #cv-gap("bullet_after_pt")
      #cv-b[RAG-basierte KI-Suche über Projekt- und Portfoliodaten entwickelt; internes Wissen durchsuchbar gemacht]
      #cv-gap("bullet_after_pt")
      #cv-b[Excel-Projektfinanzierungsmodelle erstellt; Cashflows, Renditen und Finanzierungsszenarien analysiert]
      #cv-entry-gap()
      // ccvl-station: swisscom
      #cv-h[Cloud Strategy & Transformation: #brand[Swisscom Financial Services]]
      #cv-gap("heading_after_pt")
      #cv-s[Executive Assistant & Consultant | B2B & Infrastruktur · Jun 2024 – Mär 2025 · Bern + Zürich]
      #cv-gap("subheading_after_pt")
      #cv-b[Achtstellige Infrastrukturinvestitionen im SteerCo präsentiert; Optionen mit Senior Stakeholdern diskutiert]
      #cv-gap("bullet_after_pt")
      #cv-b[Lieferantenverhandlungen über CHF 10 Mio. begleitet; sofort CHF 100k+ Einsparpotenzial identifiziert]
      #cv-gap("bullet_after_pt")
      #cv-b[Cloud-Ökonomie und 2× Rechendichte unter DC-Limits modelliert; für TOM-Workstream ausgewählt]
      #cv-entry-gap()
      // ccvl-station: airbus
      #cv-h[AI Engineering: #brand[AIRBUS Defence & Space]]
      #cv-gap("heading_after_pt")
      #cv-s[Risk & Compliance Analyst | KI/ML-Masterarbeit · Jul 2023 – Mär 2024 · Ingolstadt]
      #cv-gap("subheading_after_pt")
      #cv-b[Sicherheitskritische Daten aus 20+ Jahren per ML ausgewertet & für Risiko- und Kostenanalysen genutzt]
      #cv-gap("bullet_after_pt")
      #cv-b[Einzelfall: Sechsstelliges Einsparpotenzial p. a.; standortübergreifende achtstellige Investition ausgelöst]
      #cv-gap("bullet_after_pt")
      #cv-b[KI-Pilot von Grund auf für 3 Fachbereiche entwickelt; Stakeholder mit Business Case überzeugt]
      #cv-entry-gap()
      // ccvl-station: covendit
      #cv-h[M&A & Corporate Finance: #brand[COVENDIT]]
      #cv-gap("heading_after_pt")
      #cv-s[Investment Banking Analyst | Werkstudent · Apr 2022 – Jun 2022 · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[Live Buy-/Sell-Side-M&A-Mandate begleitet; DCF-/Multiples-Excel-Modelle, Teaser und IMs erstellt]
      #cv-gap("bullet_after_pt")
      #cv-b[KI-Longlisting vor ChatGPT entwickelt; Target-Screening automatisiert, Recherchezeit 80 % reduziert]
      #cv-gap("bullet_after_pt")
      #cv-b[PE-Kunden zu Targets beraten; Retainer gewonnen und Rückkehrangebot auf Associate-Level erhalten]
      #cv-entry-gap()
      // ccvl-station: nexgen
      #cv-h[Strategie- & Technologieberatung: #brand[NEXGEN Business Consultants]]
      #cv-gap("heading_after_pt")
      #cv-s[Junior Consultant (Werkstudent) | Banking-IT & Regulierung · Apr 2022 – Jun 2022 · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[BAIT | MaRisk: Regeln für T+1-Settlement in Cloud-Migrationsleitfaden für Tier-1-Banking-IT übersetzt]
      #cv-gap("bullet_after_pt")
      #cv-b[ETL-Engpass für Kundenpitch diagnostiziert; Laufzeit um 99 % von 24 h auf 15 min reduziert]
      #cv-gap("bullet_after_pt")
      #cv-b[Regulatorik- und IT-Analysen für Fachbeiträge und Kundenpitches aufbereitet; Mandatsakquise unterstützt]
      #cv-entry-gap()
      // ccvl-station: consulting-venture
      #cv-h[Management- & Technologieberatung: #brand[A Softer Space & Corbet Consulting]]
      #cv-gap("heading_after_pt")
      #cv-s[Head of Business Development | Management Consultant · Jan 2018 – Jun 2023 · CH, DE, IS, UK]
      #cv-gap("subheading_after_pt")
      #cv-b[Über Trusted-Advisor-Vertrieb auf mittleren sechsstelligen Umsatz in vier europäischen Märkten skaliert]
      #cv-gap("bullet_after_pt")
      #cv-b[Management- & IT-Mandate zu Leadership, Prozessen, Cloud und DLT von Analyse bis Umsetzung geführt]
      #cv-gap("bullet_after_pt")
      #cv-b[Projekt-P&L ganzheitlich gesteuert: Akquise, Angebote, Pricing, Verträge, Budgets, Margen und Cashflow]
      #cv-entry-gap()
      // ccvl-station: student-consulting
      #cv-h[Studentische Unternehmens- & Innovationsberatung]
      #cv-gap("heading_after_pt")
      #cv-s[GREEN Finance Consulting (BDSU) | Enactus | AIESEC · 2016 – 2023 · je 2 Semester · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[GREEN: Stipendienabwicklung auf 10× Kapazität skaliert; Datenbanksystem für Roland Berger entwickelt]
      #cv-gap("bullet_after_pt")
      #cv-b[ENACTUS X: Social Venture für Wohnungslose mitaufgebaut; Jobs geschaffen und Medienresonanz erzielt]
      #cv-gap("bullet_after_pt")
      #cv-b[AIESEC: International Placements mit DAX-Unternehmen koordiniert; Talent-Prozesse per CRM digitalisiert]
      #cv-entry-gap()
      // ccvl-station: teaching-research-venture
      #cv-h[Lehre, Marktforschung & Unternehmertum]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe-Universität Frankfurt | mehrere Arbeitgeber | selbstständig · Frankfurt]
      #cv-gap("subheading_after_pt")
      #cv-b[Tutor (für 3 Jahre gewählt) & Nachhilfe: Angewandte Statistik (SPSS, Python, R) & Mathematik]
      #cv-gap("bullet_after_pt")
      #cv-b[Marktforschung: 50+ CEOs interviewt und 1'000+ Gespräche analysiert, Auswertungen & Dashboards]
      #cv-gap("bullet_after_pt")
      #cv-b[Eigene Nebentätigkeit über 16 Jahre aufgebaut und geführt; vom technischen Service bis zum eCommerce]
    ]

    #cv-pagebreak()

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.education]
      #cv-hu[Stipendien: *Studienstiftung (Top 1%) | CDI (Top 4%, vollfinanziert) | Sandvoss (MSc & BSc)*]
      #cv-entry-gap()
      // ccvl-station: executive-education
      #cv-h[Executive Education]
      #cv-gap("heading_after_pt")
      #cv-s[Collège des Ingénieurs (CDI) · Paris – München – Turin · 2024 – 2025 · Notenschnitt: A (GPA 4.0)]
      #cv-gap("subheading_after_pt")
      #cv-b[Summer School: #brand[Schwarz Digits] als Junior Consultant zum EU AI Act beraten; Implikationen bewertet]
      #cv-gap("bullet_after_pt")
      #cv-b[Case Studies: Projektfinanzierung (NPV/ROI), Szenarioanalyse & Kapitalallokation unter Unsicherheit]
      #cv-entry-gap()
      // ccvl-station: physics-degrees
      #cv-h[M.Sc. & B.Sc. Physik]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe-Universität Frankfurt · Abschluss 2024 · Note: 1,0 (DE) | 6.0 (CH) | GPA 4.0]
      #cv-gap("subheading_after_pt")
      #cv-b[Schwerpunkte: KI/ML (1,0) | High-Tech-IP (1,15) | Elektronik (1,3) | Biophysik (1,3) | Chemie (1,0)]
      #cv-gap("bullet_after_pt")
      #cv-b[Forschung: Nahinfrarotspektroskopie | Terahertz-Bildgebung | Beschleunigerphysik (LINAC)]
      #cv-entry-gap()
      // ccvl-station: psychology-degree
      #cv-h[B.Sc. Psychologie]
      #cv-gap("heading_after_pt")
      #cv-s[Goethe-Universität Frankfurt · Abschluss 2017 · Note: 1,6 (DE) | 5.6 (CH) | GPA 3.7]
      #cv-gap("subheading_after_pt")
      #cv-b[Schwerpunkte: KI/ML & Neurowissenschaften | AR/VR-Trainings | Klinische/Organisationspsychologie (1,0)]
      #cv-gap("bullet_after_pt")
      #cv-b[FIAS-Forschung (9 Mon.): Stereosehen & neuronale Abstimmung per ML modelliert; Empathie quantifiziert]
      #cv-entry-gap()
      #cv-hu[Matura (Abitur): *1,0 (DE) | 6.0 (CH) · Jahrgangsbester · Mathe-Olympiade · Schülerakademie*]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.professional_development]
      // ccvl-station: certificates
      #cv-h[Zertifikate & Weiterbildung]
      #cv-gap("heading_after_pt")
      #cv-s[Finanzen | Datenanalyse | GenAI | Leadership]
      #cv-gap("subheading_after_pt")
      #cv-b[CFI-Zertifikatsprogramme (laufend): BIDA | CBCA | CMSA | FMVA; Trainings: Excel (VBA) | BI (Tableau)]
      #cv-gap("bullet_after_pt")
      #cv-b[Weitere Trainings: GenAI | Automatisierung | Rhetorik | Verhandlung | Leadership | Kommunikation]
      #cv-entry-gap()
      // ccvl-station: consulting-finance-networks
      #cv-h[Consulting- & Finance-Netzwerke]
      #cv-gap("heading_after_pt")
      #cv-s[Marktnähe durch laufenden Austausch mit Praktikern und erfahrenen Sparringspartnern]
      #cv-gap("subheading_after_pt")
      #cv-b[Im Studium: Bain Spark | BCG Emeralds | WFI Consulting Cup; BDSU- & Studienstiftung-Alumnus]
      #cv-gap("bullet_after_pt")
      #cv-b[SECA Young Member; vernetzt mit Praktikern aus Schweizer PE, VC und Corporate Development]
      #cv-entry-gap()
      // ccvl-station: technology-communities
      #cv-h[Tech-Communities & Konferenzen]
      #cv-gap("heading_after_pt")
      #cv-s[Am Puls neuer Technologien, Werkzeuge und praktischer Anwendungen]
      #cv-gap("subheading_after_pt")
      #cv-b[Swiss Python Summit & Web Zurich (2025) mitorganisiert; AV-Betrieb und Sprecherkoordination]
      #cv-gap("bullet_after_pt")
      #cv-b[Digitale Gesellschaft | LUG | digitalswitzerland | Impact Hub; Praxisfokus: GenAI & Digitale Souveränität]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.engagement]
      // ccvl-station: crisis-support
      #cv-h[Harm Reduction & Krisenunterstützung]
      #cv-gap("heading_after_pt")
      #cv-s(min-fill: 20, target-fill: 35)[Krisenintervention & Akuthilfe]
      #cv-gap("subheading_after_pt")
      #cv-b[Mehrfach lebensrettend eingegriffen; Ersthilfe geleistet und Übergabe an Rettungskräfte sichergestellt]
      #cv-gap("bullet_after_pt")
      #cv-b[Akute psychosoziale Krisen deeskaliert; Betroffene stabilisiert, orientiert und an Fachstellen vermittelt]
      #cv-entry-gap()
      // ccvl-station: mentoring
      #cv-h[Beratung, Mentoring & Fachschaft]
      #cv-gap("heading_after_pt")
      #cv-s[Frühe digitale Jugendberatung | fachübergreifender Wissenstransfer]
      #cv-gap("subheading_after_pt")
      #cv-b[Als einer von wenigen Männern bei Kids Hotline zu Identität, Körperbild & Selbstzweifeln beraten]
      #cv-gap("bullet_after_pt")
      #cv-b[Psychologie-Mentoring für alle Jahrgänge mitentwickelt; Fachschaft Physik & Night of Science unterstützt]
    ]

    #block(breakable: false)[
      #cv-compact-heading[#cv-strings.personal]
      // ccvl-station: family-responsibility
      #cv-h[Bildungsaufstieg & Verantwortung]
      #cv-gap("heading_after_pt")
      #cv-s[First Generation Academic | Bezugsperson für Geschwister (6 bzw. 12 Jahre jünger) | Pflegezeit 2025]
      #cv-gap("subheading_after_pt")
      #cv-b[Geschwister persönlich & finanziell unterstützt: Abitur (1,0) | Medizinstudium | Unternehmensgründung]
      #cv-gap("bullet_after_pt")
      #cv-b[Pflegezeit 2025: Versorgung, Finanzierung & Langzeitpflege meiner Mutter geplant und organisiert]
      #cv-entry-gap()
      // ccvl-station: open-source-community
      #cv-h[Teilen & Mitgestalten]
      #cv-gap("heading_after_pt")
      #cv-s[Interkulturelles Zusammenleben | internationale Zusammenarbeit an Open-Source-Software]
      #cv-gap("subheading_after_pt")
      #cv-b[Mit 20+ Menschen aus 10+ Ländern zusammengelebt; interkulturellen Austausch aktiv gestaltet]
      #cv-gap("bullet_after_pt")
      #cv-b[50+ Open-Source-Projekte veröffentlicht; an weiteren mitgewirkt, zuletzt oo7 (Cybersecurity)]
    ]

    #if cv-pages >= 3 [
      #cv-pagebreak()

      #cv-superheading[#cv-strings.projects]
      #block(breakable: false)[
        #cv-spacious-heading[#cv-strings.projects_ongoing]
        // ccvl-project: product-innovation
        #cv-h[Produktinnovation & Engineering]
        #cv-gap("heading_after_pt")
        #cv-s[Produktreleases 2026: Local-First-KI | Remote Development | Systems UX]
        #cv-gap("subheading_after_pt")
        #cv-b[cfetch: lokales KI-Gedächtnis (RAG) | bis zu 93,4 % weniger Kontextballast | \>15 % Tokeneinsparpotenzial]
        #cv-gap("bullet_after_pt")
        #cv-b[cterm: Remote-First-Terminal für KI-Coding | dotkeeper: P2P-Code-Sync | cbar: Cross-Machine App Matrix]
        #cv-entry-gap()
        // ccvl-project: declarative-systems-platform
        #cv-h[Deklarative Systemplattform]
        #cv-gap("heading_after_pt")
        #cv-s[Produktreleases 2026: 50+ Systembausteine für NixOS, Arch & GCP]
        #cv-gap("subheading_after_pt")
        #cv-b[NixOS & Arch: Hosts, Storage, Netzwerk, Desktops & Apps in einer reproduzierbaren Plattform vereint]
        #cv-gap("bullet_after_pt")
        #cv-b[NixOS, k3s, Argo CD & OpenTofu auf GCP & Bare Metal ausgerollt | signierte Updates | Prüfung | Rollback]
        #cv-entry-gap()
        // ccvl-project: content-innovation
        #cv-h[Content-Innovation & KI-gestützte Medien]
        #cv-gap("heading_after_pt")
        #cv-s[Neue Formate: 4K-Video, GenAI & ausfallsicheres Audio · seit 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[4K-Mehrkamera-Videos für Swiss Python Summit, Winterkongress & CoSin (Chaos Singularity) produziert]
        #cv-gap("bullet_after_pt")
        #cv-b[ComfyUI für GenAI auf eigener GPU betrieben | caudio: 3-Host-Audio-Routing mit Failover & Recovery]
        #cv-entry-gap()
        // ccvl-project: careervector-jobcache
        #cv-h[CareerVector & JobCache]
        #cv-gap("heading_after_pt")
        #cv-s[KI-native Karriereplattform & Jobdaten-Pipeline · live seit 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[CareerVector: KI-native kollaborative Karriereplattform für Web, Desktop & Terminal mit Typst-Rendering]
        #cv-gap("bullet_after_pt")
        #cv-b[JobCache: 91 Rust-Adapter für kontinuierliche Erfassung & Deduplizierung von CH/EU-Stellenanzeigen]
        #cv-entry-gap()
        // ccvl-project: private-ai-cloud
        #cv-h[Private KI- & Cloud-Plattform]
        #cv-gap("heading_after_pt")
        #cv-s[Digitale Souveränität im Produktivbetrieb für 10+ Nutzer · seit 2024]
        #cv-gap("subheading_after_pt")
        #cv-b[30+ Dienste und 100+ TB mit SSO, Monitoring, Backups & Disaster Recovery end-to-end betrieben]
        #cv-gap("bullet_after_pt")
        #cv-b[Lokale LLMs & KI-Agenten auf eigener GPU-Infrastruktur betrieben | \>90 % günstiger als Public Cloud]
      ]

      #block(breakable: false)[
        #cv-spacious-heading[#cv-strings.projects_delivered]
        // ccvl-project: management-buy-in
        #cv-h[Management Buy-In: Deal Origination & Due Diligence]
        #cv-gap("heading_after_pt")
        #cv-s[Indischer IT-Outsourcer · eigenständiges MBI bis zum Kaufentscheid · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Indischen IT-Outsourcer als MBI-Ziel identifiziert und End-to-End Due Diligence eigenständig durchgeführt]
        #cv-gap("bullet_after_pt")
        #cv-b[Akquisitionsthese entwickelt | strategischen Fit, Chancen & Risiken bis zum finalen Go/No-Go bewertet]
        #cv-entry-gap()
        // ccvl-project: solar-recovery
        #cv-h[Solar-KMU: Incident Recovery & Cloud-Migration]
        #cv-gap("heading_after_pt")
        #cv-s[Betriebskritische Systeme für Vertrieb & Felddienst · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Kernsystem am ersten Tag wiederhergestellt; Vertrieb und Felddienst bis zur Ablösung arbeitsfähig gehalten]
        #cv-gap("bullet_after_pt")
        #cv-b[Cloud-Optionen gegen Betriebsanforderungen geprüft; sechs- bis siebenstellige Fehlinvestition vermieden]
        #cv-entry-gap()
        // ccvl-project: leadership-digital-pivot
        #cv-h[Leadership Advisory: Digitaler Pivot]
        #cv-gap("heading_after_pt")
        #cv-s[Frankfurter Leadership-Marke · Hybridformat & neue Vertriebskanäle · 2022]
        #cv-gap("subheading_after_pt")
        #cv-b[Performance-Leadership-Angebot für hybride Delivery überarbeitet und On-Demand-Infrastruktur aufgebaut]
        #cv-gap("bullet_after_pt")
        #cv-b[Funnel an Kundenpainpoints ausgerichtet; Umsatz diversifiziert und Kurse bei Haufe Akademie platziert]
        #cv-entry-gap()
        // ccvl-project: crypto-infrastructure
        #cv-h[Krypto-Infrastruktur: Business Case & Betrieb]
        #cv-gap("heading_after_pt")
        #cv-s[Mining-Piloten vom Business Case bis zum Betrieb · mehrere Kunden · 2021]
        #cv-gap("subheading_after_pt")
        #cv-b[Pilot und Betriebsmodell dimensioniert und kalkuliert; Hardware beschafft und operative Risiken gesteuert]
        #cv-gap("bullet_after_pt")
        #cv-b[Mining-Betrieb termingerecht, stabil und mit Monitoring aufgebaut; Hashrate via Custom-Firmware optimiert]
        #cv-entry-gap()
        // ccvl-project: it-services-ecommerce
        #cv-h[IT-Services & automatisierter eCommerce]
        #cv-gap("heading_after_pt")
        #cv-s[Eigenes Geschäft · Exit zu 80 % des Buchwerts · 2009 – 2025]
        #cv-gap("subheading_after_pt")
        #cv-b[Hardwarehandel, Diagnose, Custom Builds & Reparaturen für KMU/B2C-Kunden aufgebaut & betrieben]
        #cv-gap("bullet_after_pt")
        #cv-b[Listing, Bestand, Tracking und Logistik des fünfstelligen eBay-Betriebs per Mini-ERP automatisiert]
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
        #cv-h[AI-Produkte, Tooling & Modellökosysteme]
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
        #cv-b[Data Science & ML: Statistik | Machine Learning | Zeitreihen | Predictive Modelling | Experimente | R]
        #cv-entry-gap()
        // ccvl-competency: software-infrastructure
        #cv-h[Software Engineering, Web & Plattformen]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Engineering: Python | Rust | Go | Java | TypeScript | JavaScript | Bash | SQL | Git | CI/CD | Testing]
        #cv-gap("bullet_after_pt")
        #cv-b[Web & Publishing: Svelte | Astro | HTML | CSS | REST | GraphQL | WebSockets | Markdown | Typst]
        #cv-gap("bullet_after_pt")
        #cv-b[Cloud & Data Platforms: PostgreSQL | Data Pipelines | Linux | Nix/NixOS | Kubernetes | GitOps | OpenTofu]
        #cv-spacious-heading[#cv-strings.pillar_strategy]
        // ccvl-competency: innovation-management
        #cv-h[Innovationsmanagement & Emerging Technologies]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Innovation Management: Innovation Pipeline | Stage-Gate | Incremental Innovation | Disruptive Innovation]
        #cv-gap("bullet_after_pt")
        #cv-b[Technology Scouting: Emerging Technologies | Trendanalyse | Horizon Scanning | Technologiebewertung]
        #cv-gap("bullet_after_pt")
        #cv-b[Produktinnovation: Product Discovery | Prototyping | Proof of Concept (PoC) | MVP | Marktvalidierung]
        #cv-entry-gap()
        // ccvl-competency: strategy
        #cv-h[Unternehmens-, Wachstums- & Technologiestrategie]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Corporate Strategy: Strategische Planung | Szenarioplanung | Wettbewerbsanalyse | Decision Support]
        #cv-gap("bullet_after_pt")
        #cv-b[Growth Strategy: Business Development | Markteintritt | Go-to-Market | Partnerschaften | Pricing | B2B]
        #cv-gap("bullet_after_pt")
        #cv-b[Technology Strategy: AI Strategy | Roadmaps | Business Cases | Enterprise Architecture | TCO | FinOps]
        #cv-entry-gap()
        // ccvl-competency: transformation-governance
        #cv-h[Transformation, Operating Models & Governance]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Operating Models: Target Operating Model (TOM) | Organisationsdesign | Decision Rights | Rollendesign]
        #cv-gap("bullet_after_pt")
        #cv-b[Change & Value Creation: AI Adoption | Change Management | Benefits Realisation | Cost Transformation]
        #cv-gap("bullet_after_pt")
        #cv-b[AI Governance: EU AI Act | Responsible AI | Model Risk | DORA | Operational Resilience | DSGVO]
        #cv-spacious-heading[#cv-strings.pillar_finance]
        // ccvl-competency: finance-ma
        #cv-h[Finance Transformation, Corporate Finance & M&A]
        #cv-gap("competency_heading_after_pt")
        #cv-b[CFO Agenda: Finance Platform | Finance Data Architecture | Planung & Forecasting | AI-enabled Finance]
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
        #cv-h[Trading, Quantitative Finance & Risiko]
        #cv-gap("competency_heading_after_pt")
        #cv-b[Energy & Commodity Markets: Power Trading | Day-Ahead | Intraday | Gas/LNG | Metals | Carbon | Freight]
        #cv-gap("bullet_after_pt")
        #cv-b[Systematic Trading: Alpha Signals | Backtesting | Trade Execution | Futures | Swaps | Options | Hedging]
        #cv-gap("bullet_after_pt")
        #cv-b[Quantitative Risk: PnL | Value at Risk (VaR) | Stress Testing | Monte Carlo | Optionsbewertung | Volatilität]
      ]
    ]
  ]
}
