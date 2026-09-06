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
#let application-path = sys.inputs.at("application", default: "/cvl/de-ch/application.toml")
#let application = toml(application-path)
#validate-application(application, expected-language: "de-CH", require-cv: true)

// Style axis: the explicit `style` input injected by render.rs (resolved from
// options.style) wins; a manual render without it falls back to the record,
// then to the harvard default. Whitespace below comes from that style's TOML
// knobs, never from forked literals.
#let style-input = sys.inputs.at("style", default: "")
#let style-name = if style-input != "" { style-input } else { application.options.at("style", default: "harvard") }
#let style = load-style(style-name)
#show: document-style.with(locale: "de-ch", style: style)

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
  let localized = localized-profile.at("de-ch")
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
  #cv-compact-heading[Erfahrung]
  // ccvl-station: cenvion
  #cv-h[Infrastructure Investments & Asset Management: #brand[CENVION]]
  #v(cv-heading-after)
  #cv-s[Associate Intern | Infrastructure Investments · Jan 2026 – Mär 2026 (plus freie Mitarbeit) · Wollerau (CH)]
  #v(cv-subheading-after)
  #cv-b[#brand[Claude] für Investment Reporting eingeführt; GenAI in Analyse- und Berichtsworkflows des Teams integriert]
  #v(cv-bullet-after)
  #cv-b[RAG-basierte KI-Suche über Projekt- und Portfoliodaten entwickelt; internes Wissen durchsuchbar gemacht]
  #v(cv-bullet-after)
  #cv-b[Excel-Projektfinanzierungsmodelle erstellt; Cashflows, Renditen und Finanzierungsszenarien analysiert]
  #cv-entry-gap()
  // ccvl-station: swisscom
  #cv-h[Cloud Strategy & Transformation: #brand[Swisscom Financial Services]]
  #v(cv-heading-after)
  #cv-s[Executive Assistant & Consultant | B2B & Infrastruktur · Jun 2024 – Mär 2025 · Bern + Zürich]
  #v(cv-subheading-after)
  #cv-b[Achtstellige Infrastrukturinvestitionen im SteerCo präsentiert; Optionen mit Senior Stakeholdern diskutiert]
  #v(cv-bullet-after)
  #cv-b[Lieferantenverhandlungen über CHF 10 Mio. begleitet; sofort CHF 100k+ Einsparpotenzial identifiziert]
  #v(cv-bullet-after)
  #cv-b[Cloud-Ökonomie und 2× Rechendichte unter DC-Limits modelliert; für TOM-Workstream ausgewählt]
  #cv-entry-gap()
  // ccvl-station: airbus
  #cv-h[AI Engineering: #brand[AIRBUS Defence & Space]]
  #v(cv-heading-after)
  #cv-s[Risk & Compliance Analyst | KI/ML-Masterarbeit · Jul 2023 – Mär 2024 · Ingolstadt]
  #v(cv-subheading-after)
  #cv-b[Sicherheitskritische Daten aus 20+ Jahren per ML ausgewertet & für Risiko- und Kostenanalysen genutzt]
  #v(cv-bullet-after)
  #cv-b[Einzelfall: Sechsstelliges Einsparpotenzial p. a.; standortübergreifende achtstellige Investition ausgelöst]
  #v(cv-bullet-after)
  #cv-b[KI-Pilot von Grund auf für 3 Fachbereiche entwickelt; Stakeholder mit Business Case überzeugt]
  #cv-entry-gap()
  // ccvl-station: covendit
  #cv-h[M&A & Corporate Finance: #brand[COVENDIT]]
  #v(cv-heading-after)
  #cv-s[Investment Banking Analyst | Werkstudent · Apr 2022 – Jun 2022 · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[Live Buy-/Sell-Side-M&A-Mandate begleitet; DCF-/Multiples-Excel-Modelle, Teaser und IMs erstellt]
  #v(cv-bullet-after)
  #cv-b[KI-Longlisting vor ChatGPT entwickelt; Target-Screening automatisiert, Recherchezeit 80 % reduziert]
  #v(cv-bullet-after)
  #cv-b[PE-Kunden zu Targets beraten; Retainer gewonnen und Rückkehrangebot auf Associate-Level erhalten]
  #cv-entry-gap()
  // ccvl-station: nexgen
  #cv-h[Strategie- & Technologieberatung: #brand[NEXGEN Business Consultants]]
  #v(cv-heading-after)
  #cv-s[Junior Consultant (Werkstudent) | Banking-IT & Regulierung · Apr 2022 – Jun 2022 · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[BAIT | MaRisk: Regeln für T+1-Settlement in Cloud-Migrationsleitfaden für Tier-1-Banking-IT übersetzt]
  #v(cv-bullet-after)
  #cv-b[ETL-Engpass für Kundenpitch diagnostiziert; Laufzeit um 99 % von 24 h auf 15 min reduziert]
  #v(cv-bullet-after)
  #cv-b[Regulatorik- und IT-Analysen für Fachbeiträge und Kundenpitches aufbereitet; Mandatsakquise unterstützt]
  #cv-entry-gap()
  // ccvl-station: consulting-venture
  #cv-h[Management- & Technologieberatung: #brand[A Softer Space & Corbet Consulting]]
  #v(cv-heading-after)
  #cv-s[Head of Business Development | Management Consultant · Jan 2018 – Jun 2023 · CH, DE, IS, UK]
  #v(cv-subheading-after)
  #cv-b[Über Trusted-Advisor-Vertrieb auf mittleren sechsstelligen Umsatz in vier europäischen Märkten skaliert]
  #v(cv-bullet-after)
  #cv-b[Management- & IT-Mandate zu Leadership, Prozessen, Cloud und DLT von Analyse bis Umsetzung geführt]
  #v(cv-bullet-after)
  #cv-b[Projekt-P&L ganzheitlich gesteuert: Akquise, Angebote, Pricing, Verträge, Budgets, Margen und Cashflow]
  #cv-entry-gap()
  // ccvl-station: student-consulting
  #cv-h[Studentische Unternehmens- & Innovationsberatung]
  #v(cv-heading-after)
  #cv-s[GREEN Finance Consulting (BDSU) | Enactus | AIESEC · 2016 – 2023 · je 2 Semester · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[GREEN: Stipendienabwicklung auf 10× Kapazität skaliert; Datenbanksystem für Roland Berger entwickelt]
  #v(cv-bullet-after)
  #cv-b[ENACTUS X: Social Venture für Wohnungslose mitaufgebaut; Jobs geschaffen und Medienresonanz erzielt]
  #v(cv-bullet-after)
  #cv-b[AIESEC: International Placements mit DAX-Unternehmen koordiniert; Talent-Prozesse per CRM digitalisiert]
  #cv-entry-gap()
  // ccvl-station: teaching-research-venture
  #cv-h[Lehre, Marktforschung & Unternehmertum]
  #v(cv-heading-after)
  #cv-s[Goethe-Universität Frankfurt | mehrere Arbeitgeber | selbstständig · Frankfurt]
  #v(cv-subheading-after)
  #cv-b[Tutor (für 3 Jahre gewählt) & Nachhilfe: Angewandte Statistik (SPSS, Python, R) & Mathematik]
  #v(cv-bullet-after)
  #cv-b[Marktforschung: 50+ CEOs interviewt und 1.000+ Gespräche analysiert, Auswertungen & Dashboards]
  #v(cv-bullet-after)
  #cv-b[Eigene Nebentätigkeit über 16 Jahre aufgebaut und geführt; vom technischen Service bis zum eCommerce]
]

#cv-pagebreak()

#block(breakable: false)[
  #cv-compact-heading[Bildung]
  #cv-hu[Stipendien: *Studienstiftung (Top 1%) | CDI (Top 4%, vollfinanziert) | Sandvoss (MSc & BSc)*]
  #cv-entry-gap()
  // ccvl-station: executive-education
  #cv-h[Executive Education]
  #v(cv-heading-after)
  #cv-s[Collège des Ingénieurs (CDI) · Paris – München – Turin · 2024 – 2025 · Notenschnitt: A (GPA 4.0)]
  #v(cv-subheading-after)
  #cv-b[Summer School: #brand[Schwarz Digits] als Junior Consultant zum EU AI Act beraten; Implikationen bewertet]
  #v(cv-bullet-after)
  #cv-b[Case Studies: Projektfinanzierung (NPV/ROI), Szenarioanalyse & Kapitalallokation unter Unsicherheit]
  #cv-entry-gap()
  // ccvl-station: physics-degrees
  #cv-h[M.Sc. & B.Sc. Physik]
  #v(cv-heading-after)
  #cv-s[Goethe-Universität Frankfurt · Abschluss 2024 · Note: 1,0 (DE) | 6,0 (CH) | GPA 4.0]
  #v(cv-subheading-after)
  #cv-b[Schwerpunkte: KI/ML (1,0) | High-Tech-IP (1,15) | Elektronik (1,3) | Biophysik (1,3) | Chemie (1,0)]
  #v(cv-bullet-after)
  #cv-b[Forschung: Nahinfrarotspektroskopie | Terahertz-Bildgebung | Beschleunigerphysik (LINAC)]
  #cv-entry-gap()
  // ccvl-station: psychology-degree
  #cv-h[B.Sc. Psychologie]
  #v(cv-heading-after)
  #cv-s[Goethe-Universität Frankfurt · Abschluss 2017 · Note: 1,6 (DE) | 5,6 (CH) | GPA 3.7]
  #v(cv-subheading-after)
  #cv-b[Schwerpunkte: KI/ML & Neurowissenschaften | AR/VR-Trainings | Klinische/Organisationspsychologie (1,0)]
  #v(cv-bullet-after)
  #cv-b[FIAS-Forschung (9 Mon.): Stereosehen & neuronale Abstimmung per ML modelliert; Empathie quantifiziert]
  #cv-entry-gap()
  #cv-hu[Matura (Abitur): *1,0 (DE) | 6,0 (CH) · Jahrgangsbester · Mathe-Olympiade · Schülerakademie*]
]

#block(breakable: false)[
  #cv-compact-heading[Professional Development]
  // ccvl-station: certificates
  #cv-h[Zertifikate & Weiterbildung]
  #v(cv-heading-after)
  #cv-s[Finanzen | Datenanalyse | GenAI | Leadership]
  #v(cv-subheading-after)
  #cv-b[CFI-Zertifikatsprogramme (laufend): BIDA | CBCA | CMSA | FMVA; Trainings: Excel (VBA) | BI (Tableau)]
  #v(cv-bullet-after)
  #cv-b[Weitere Trainings: GenAI | Automatisierung | Rhetorik | Verhandlung | Leadership | Kommunikation]
  #cv-entry-gap()
  // ccvl-station: consulting-finance-networks
  #cv-h[Consulting- & Finance-Netzwerke]
  #v(cv-heading-after)
  #cv-s[Marktnähe durch laufenden Austausch mit Praktikern und erfahrenen Sparringspartnern]
  #v(cv-subheading-after)
  #cv-b[Im Studium: Bain Spark | BCG Emeralds | WFI Consulting Cup; BDSU- & Studienstiftung-Alumnus]
  #v(cv-bullet-after)
  #cv-b[SECA Young Member; vernetzt mit Praktikern aus Schweizer PE, VC und Corporate Development]
  #cv-entry-gap()
  // ccvl-station: technology-communities
  #cv-h[Tech-Communities & Konferenzen]
  #v(cv-heading-after)
  #cv-s[Am Puls neuer Technologien, Werkzeuge und praktischer Anwendungen]
  #v(cv-subheading-after)
  #cv-b[Swiss Python Summit & Web Zurich (2025) mitorganisiert; AV-Betrieb und Sprecherkoordination]
  #v(cv-bullet-after)
  #cv-b[Digitale Gesellschaft | LUG | digitalswitzerland | Impact Hub; Praxisfokus: GenAI & Digitale Souveränität]
]

#block(breakable: false)[
  #cv-compact-heading[Engagement]
  // ccvl-station: crisis-support
  #cv-h[Harm Reduction & Krisenunterstützung]
  #v(cv-heading-after)
  #cv-s(min-fill: 20, target-fill: 35)[Krisenintervention & Akuthilfe]
  #v(cv-subheading-after)
  #cv-b[Mehrfach lebensrettend eingegriffen; Ersthilfe geleistet und Übergabe an Rettungskräfte sichergestellt]
  #v(cv-bullet-after)
  #cv-b[Akute psychosoziale Krisen deeskaliert; Betroffene stabilisiert, orientiert und an Fachstellen vermittelt]
  #cv-entry-gap()
  // ccvl-station: mentoring
  #cv-h[Beratung, Mentoring & Fachschaft]
  #v(cv-heading-after)
  #cv-s[Frühe digitale Jugendberatung | fachübergreifender Wissenstransfer]
  #v(cv-subheading-after)
  #cv-b[Als einer von wenigen Männern bei Kids Hotline zu Identität, Körperbild & Selbstzweifeln beraten]
  #v(cv-bullet-after)
  #cv-b[Psychologie-Mentoring für alle Jahrgänge mitentwickelt; Fachschaft Physik & Night of Science unterstützt]
]

#block(breakable: false)[
  #cv-compact-heading[Persönliches]
  // ccvl-station: family-responsibility
  #cv-h[Bildungsaufstieg & Verantwortung]
  #v(cv-heading-after)
  #cv-s[First Generation Academic | Bezugsperson für Geschwister (6 bzw. 12 Jahre jünger) | Pflegezeit 2025]
  #v(cv-subheading-after)
  #cv-b[Geschwister persönlich & finanziell unterstützt: Abitur (1,0) | Medizinstudium | Unternehmensgründung]
  #v(cv-bullet-after)
  #cv-b[Pflegezeit 2025: Versorgung, Finanzierung & Langzeitpflege meiner Mutter geplant und organisiert]
  #cv-entry-gap()
  // ccvl-station: open-source-community
  #cv-h[Teilen & Mitgestalten]
  #v(cv-heading-after)
  #cv-s[Interkulturelles Zusammenleben | internationale Zusammenarbeit an Open-Source-Software]
  #v(cv-subheading-after)
  #cv-b[Mit 20+ Menschen aus 10+ Ländern zusammengelebt; interkulturellen Austausch aktiv gestaltet]
  #v(cv-bullet-after)
  #cv-b[50+ Open-Source-Projekte veröffentlicht; an weiteren mitgewirkt, zuletzt oo7 (Cybersecurity)]
]

#if cv-pages >= 3 [
  #cv-pagebreak()

  #cv-superheading[Projekte & Initiativen]
  #block(breakable: false)[
    #cv-spacious-heading[Laufend]
    // ccvl-project: product-innovation
    #cv-h[Produktinnovation & Engineering]
    #v(cv-heading-after)
    #cv-s[Produktreleases 2026: Local-First-KI | Remote Development | Systems UX]
    #v(cv-subheading-after)
    #cv-b[cfetch: lokales KI-Gedächtnis (RAG) | bis zu 93,4 % weniger Kontextballast | \>15 % Tokeneinsparpotenzial]
    #v(cv-bullet-after)
    #cv-b[cterm: Remote-First-Terminal für KI-Coding | dotkeeper: P2P-Code-Sync | cbar: Cross-Machine App Matrix]
    #cv-entry-gap()
    // ccvl-project: declarative-systems-platform
    #cv-h[Deklarative Systemplattform]
    #v(cv-heading-after)
    #cv-s[Produktreleases 2026: 50+ Systembausteine für NixOS, Arch & GCP]
    #v(cv-subheading-after)
    #cv-b[NixOS & Arch: Hosts, Storage, Netzwerk, Desktops & Apps in einer reproduzierbaren Plattform vereint]
    #v(cv-bullet-after)
    #cv-b[NixOS, k3s, Argo CD & OpenTofu auf GCP & Bare Metal ausgerollt | signierte Updates | Prüfung | Rollback]
    #cv-entry-gap()
    // ccvl-project: content-innovation
    #cv-h[Content-Innovation & KI-gestützte Medien]
    #v(cv-heading-after)
    #cv-s[Neue Formate: 4K-Video, GenAI & ausfallsicheres Audio · seit 2025]
    #v(cv-subheading-after)
    #cv-b[4K-Mehrkamera-Videos für Swiss Python Summit, Winterkongress & CoSin (Chaos Singularity) produziert]
    #v(cv-bullet-after)
    #cv-b[ComfyUI für GenAI auf eigener GPU betrieben | caudio: 3-Host-Audio-Routing mit Failover & Recovery]
    #cv-entry-gap()
    // ccvl-project: careervector-jobcache
    #cv-h[CareerVector & JobCache]
    #v(cv-heading-after)
    #cv-s[KI-native Karriereplattform & Jobdaten-Pipeline · live seit 2025]
    #v(cv-subheading-after)
    #cv-b[CareerVector: KI-native kollaborative Karriereplattform für Web, Desktop & Terminal mit Typst-Rendering]
    #v(cv-bullet-after)
    #cv-b[JobCache: 91 Rust-Adapter für kontinuierliche Erfassung & Deduplizierung von CH/EU-Stellenanzeigen]
    #cv-entry-gap()
    // ccvl-project: private-ai-cloud
    #cv-h[Private KI- & Cloud-Plattform]
    #v(cv-heading-after)
    #cv-s[Digitale Souveränität im Produktivbetrieb für 10+ Nutzer · seit 2024]
    #v(cv-subheading-after)
    #cv-b[30+ Dienste und 100+ TB mit SSO, Monitoring, Backups & Disaster Recovery end-to-end betrieben]
    #v(cv-bullet-after)
    #cv-b[Lokale LLMs & KI-Agenten auf eigener GPU-Infrastruktur betrieben | \>90 % günstiger als Public Cloud]
  ]

  #block(breakable: false)[
    #cv-spacious-heading[Realisiert]
    // ccvl-project: management-buy-in
    #cv-h[Management Buy-In: Deal Origination & Due Diligence]
    #v(cv-heading-after)
    #cv-s[Indischer IT-Outsourcer · eigenständiges MBI bis zum Kaufentscheid · 2022]
    #v(cv-subheading-after)
    #cv-b[Indischen IT-Outsourcer als MBI-Ziel identifiziert und End-to-End Due Diligence eigenständig durchgeführt]
    #v(cv-bullet-after)
    #cv-b[Akquisitionsthese entwickelt | strategischen Fit, Chancen & Risiken bis zum finalen Go/No-Go bewertet]
    #cv-entry-gap()
    // ccvl-project: solar-recovery
    #cv-h[Solar-KMU: Incident Recovery & Cloud-Migration]
    #v(cv-heading-after)
    #cv-s[Betriebskritische Systeme für Vertrieb & Felddienst · 2022]
    #v(cv-subheading-after)
    #cv-b[Kernsystem am ersten Tag wiederhergestellt; Vertrieb und Felddienst bis zur Ablösung arbeitsfähig gehalten]
    #v(cv-bullet-after)
    #cv-b[Cloud-Optionen gegen Betriebsanforderungen geprüft; sechs- bis siebenstellige Fehlinvestition vermieden]
    #cv-entry-gap()
    // ccvl-project: leadership-digital-pivot
    #cv-h[Leadership Advisory: Digitaler Pivot]
    #v(cv-heading-after)
    #cv-s[Frankfurter Leadership-Marke · Hybridformat & neue Vertriebskanäle · 2022]
    #v(cv-subheading-after)
    #cv-b[Performance-Leadership-Angebot für hybride Delivery überarbeitet und On-Demand-Infrastruktur aufgebaut]
    #v(cv-bullet-after)
    #cv-b[Funnel an Kundenpainpoints ausgerichtet; Umsatz diversifiziert und Kurse bei Haufe Akademie platziert]
    #cv-entry-gap()
    // ccvl-project: crypto-infrastructure
    #cv-h[Krypto-Infrastruktur: Business Case & Betrieb]
    #v(cv-heading-after)
    #cv-s[Mining-Piloten vom Business Case bis zum Betrieb · mehrere Kunden · 2021]
    #v(cv-subheading-after)
    #cv-b[Pilot und Betriebsmodell dimensioniert und kalkuliert; Hardware beschafft und operative Risiken gesteuert]
    #v(cv-bullet-after)
    #cv-b[Mining-Betrieb termingerecht, stabil und mit Monitoring aufgebaut; Hashrate via Custom-Firmware optimiert]
    #cv-entry-gap()
    // ccvl-project: it-services-ecommerce
    #cv-h[IT-Services & automatisierter eCommerce]
    #v(cv-heading-after)
    #cv-s[Eigenes Geschäft · Exit zu 80 % des Buchwerts · 2009 – 2025]
    #v(cv-subheading-after)
    #cv-b[Hardwarehandel, Diagnose, Custom Builds & Reparaturen für KMU/B2C-Kunden aufgebaut & betrieben]
    #v(cv-bullet-after)
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

  #cv-superheading[Kompetenzen & KI Keywords]
  #block(breakable: false)[
    #cv-spacious-heading[AI, Software & Data]
    // ccvl-competency: ai-products-tooling
    #cv-h[AI-Produkte, Tooling & Modellökosysteme]
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
    #cv-b[Data Science & ML: Statistik | Machine Learning | Zeitreihen | Predictive Modelling | Experimente | R]
    #cv-entry-gap()
    // ccvl-competency: software-infrastructure
    #cv-h[Software Engineering, Web & Plattformen]
    #v(cv-competency-heading-after)
    #cv-b[Engineering: Python | Rust | Go | Java | TypeScript | JavaScript | Bash | SQL | Git | CI/CD | Testing]
    #v(cv-bullet-after)
    #cv-b[Web & Publishing: Svelte | Astro | HTML | CSS | REST | GraphQL | WebSockets | Markdown | Typst]
    #v(cv-bullet-after)
    #cv-b[Cloud & Data Platforms: PostgreSQL | Data Pipelines | Linux | Nix/NixOS | Kubernetes | GitOps | OpenTofu]
    #cv-spacious-heading[Strategie, Innovation & Transformation]
    // ccvl-competency: innovation-management
    #cv-h[Innovationsmanagement & Emerging Technologies]
    #v(cv-competency-heading-after)
    #cv-b[Innovation Management: Innovation Pipeline | Stage-Gate | Incremental Innovation | Disruptive Innovation]
    #v(cv-bullet-after)
    #cv-b[Technology Scouting: Emerging Technologies | Trendanalyse | Horizon Scanning | Technologiebewertung]
    #v(cv-bullet-after)
    #cv-b[Produktinnovation: Product Discovery | Prototyping | Proof of Concept (PoC) | MVP | Marktvalidierung]
    #cv-entry-gap()
    // ccvl-competency: strategy
    #cv-h[Unternehmens-, Wachstums- & Technologiestrategie]
    #v(cv-competency-heading-after)
    #cv-b[Corporate Strategy: Strategische Planung | Szenarioplanung | Wettbewerbsanalyse | Decision Support]
    #v(cv-bullet-after)
    #cv-b[Growth Strategy: Business Development | Markteintritt | Go-to-Market | Partnerschaften | Pricing | B2B]
    #v(cv-bullet-after)
    #cv-b[Technology Strategy: AI Strategy | Roadmaps | Business Cases | Enterprise Architecture | TCO | FinOps]
    #cv-entry-gap()
    // ccvl-competency: transformation-governance
    #cv-h[Transformation, Operating Models & Governance]
    #v(cv-competency-heading-after)
    #cv-b[Operating Models: Target Operating Model (TOM) | Organisationsdesign | Decision Rights | Rollendesign]
    #v(cv-bullet-after)
    #cv-b[Change & Value Creation: AI Adoption | Change Management | Benefits Realisation | Cost Transformation]
    #v(cv-bullet-after)
    #cv-b[AI Governance: EU AI Act | Responsible AI | Model Risk | DORA | Operational Resilience | DSGVO]
    #cv-spacious-heading[Finanzen, Investments & Märkte]
    // ccvl-competency: finance-ma
    #cv-h[Finance Transformation, Corporate Finance & M&A]
    #v(cv-competency-heading-after)
    #cv-b[CFO Agenda: Finance Platform | Finance Data Architecture | Planung & Forecasting | AI-enabled Finance]
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
    #cv-h[Trading, Quantitative Finance & Risiko]
    #v(cv-competency-heading-after)
    #cv-b[Energy & Commodity Markets: Power Trading | Day-Ahead | Intraday | Gas/LNG | Metals | Carbon | Freight]
    #v(cv-bullet-after)
    #cv-b[Systematic Trading: Alpha Signals | Backtesting | Trade Execution | Futures | Swaps | Options | Hedging]
    #v(cv-bullet-after)
    #cv-b[Quantitative Risk: PnL | Value at Risk (VaR) | Stress Testing | Monte Carlo | Optionsbewertung | Volatilität]
  ]
]

#assert-page-count(cv-pages)
