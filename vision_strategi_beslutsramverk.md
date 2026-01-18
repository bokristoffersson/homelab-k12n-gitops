
# Vision, strategi och beslutsramverk för arkitektur

Detta dokument sammanfattar min riktning som arkitekt, min strategi för nästa steg,
samt ett konkret ramverk för hur arkitekturbeslut fattas i praktiken.

Syftet är att skapa **tydlighet i tekniska vägval**, minska framtida risk och
göra organisationen mer självständig över tid.

---

## Min riktning

Jag rör mig från att vara en arkitekt som främst **löser problem**
till att vara en arkitekt som **sätter teknisk riktning och fattar genomtänkta vägval**
i situationer där osäkerheten är hög.

> **Jag arbetar med tekniska vägval som gör framtida förändring enklare – för både teknik och människor.**

---

## Vision (konkret och användbar)

> **Jag vill vara arkitekten som hjälper organisationer att ta sig igenom tvingande
> teknikförändringar genom att skapa tydlig teknisk riktning, minska framtida risk
> och göra team mer självständiga.**

Visionen ska:
- fungera att säga högt
- förklara varför arkitektur behövs
- gå att leva med över tid

---

## Strategi i korthet

Strategi handlar om **medvetna val**, inte planer.

> **Jag använder mitt nuvarande uppdrag för att träna, bevisa och förfina ett arbetssätt
> som jag senare erbjuder externt.**

Allt jag gör nu ska:
- bygga omdöme
- skapa gemensamt språk
- reducera framtida osäkerhet

---

## Tre strategiska spår

### 1. Nuvarande jobb – träningsarena

**Syfte:** bygga legitimitet och metod, inte bara leverans.

Fokus:
- ta ägarskap för problemformulering och tekniska vägval
- använda referensarkitektur för att skapa riktning
- behandla teknikbyten (t.ex. BizTalk) som arkitektur- och beslutsfrågor

Medvetna avgränsningar:
- inte bli flaskhals
- inte äga alla detaljer
- inte optimera för perfektion

---

### 2. Metod – det jag faktiskt säljer

När jag kliver in i en organisation bidrar jag med:

1. Struktur för att identifiera viktiga tekniska beslut  
2. Stöd i osäkra vägval  
3. Minskad långsiktig risk och teknisk skuld  

Målet är alltid:
> att organisationen ska fatta bättre beslut även utan min närvaro

---

### 3. Firma & hemsida – låg intensitet, hög signal

Syftet är positionering, inte volymförsäljning.

Hemsidan ska visa:
- hur jag tänker
- vilka principer jag står för
- hur jag ser på teknikförändring och risk

Den ska få rätt personer att känna:
> “Den här personen hjälper oss att tänka klarare.”

---

## Från “välja rätt problem” till bättre språk

Uttrycket *välja rätt problem* används internt som kompass,
men utåt används hellre formuleringar som:

- sätta teknisk riktning
- göra genomtänkta tekniska vägval
- arbeta med beslut som är dyra att göra fel
- minska teknisk och organisatorisk risk över tid
- göra framtida förändring billigare

Gemensam innebörd:
fokus på orsaker, konsekvenser och långsiktiga beslut – inte symptom.

---

## Hur vi fattar arkitekturbeslut

Arkitektur handlar i praktiken om **beslut under osäkerhet**.
Alla beslut är inte lika viktiga och ska inte behandlas likadant.

### Typ 1-beslut (svåra att backa)

**Kännetecken:**
- stora, långsiktiga konsekvenser
- dyra eller smärtsamma att ändra
- påverkar många team eller verksamhetsdelar
- ofta kopplade till plattformar, ansvarsfördelning eller grundläggande mönster

**Exempel:**
- val av integrationsstrategi
- ansvarsfördelning mellan plattform och produktteam
- införande av ny gemensam plattform

**Hur de hanteras:**
- mer analys
- fler perspektiv
- tydlig dokumentation (ADR)
- beslut fattas medvetet, även om informationen är ofullständig

---

### Typ 2-beslut (lätta att ändra)

**Kännetecken:**
- lokala konsekvenser
- går relativt enkelt att justera eller rulla tillbaka
- påverkar få team
- låg långsiktig låsning

**Exempel:**
- val av bibliotek eller ramverk
- intern implementationsteknik
- detaljer i CI/CD-pipeline

**Hur de hanteras:**
- fattas snabbt
- delegeras till team
- optimeras för lärande

---

## Snabba, approximativa beslut och agilt arbete

Att fatta **snabba, approximativa beslut** står inte i konflikt med agilt arbete.
Tvärtom är det ofta en förutsättning.

### Viktig distinktion

Agilt arbete förespråkar:
> *“ta irreversibla beslut så sent som möjligt”*

Det betyder inte:
- att alla beslut ska skjutas upp
- att osäkerhet ska ignoreras

Det betyder:
- att **typ 1-beslut kräver större eftertanke**
- medan **typ 2-beslut bör fattas tidigt för att skapa lärande**

### Approximativa beslut i praktiken

Ett snabbt, approximativt beslut:
- är medvetet ofullständigt
- är öppet för att ändras
- syftar till att minska osäkerhet genom handling

I arkitektur innebär det:
- spikes istället för fullskalig design
- proof-of-concepts för att testa antaganden
- tidiga beslut om riktning, utan att låsa implementation

Detta gör att:
- osäkerhet reduceras tidigare
- feedback fås snabbare
- sena, dyra överraskningar undviks

---

## Kompass

När prioriteringar blir oklara lutar jag mig mot:

> **“Jag lägger arkitektens tid där ett felbeslut är dyrt att backa – och låter resten
> optimeras för lärande.”**

---

## Nästa steg

### Inom 30 dagar
- formulera 5 arkitekturprinciper jag kan stå för
- koppla dem till typ 1-beslut
- använda språket kring vägval aktivt i möten

### Inom 90 dagar
- ha en referensarkitektur som styr diskussioner
- ha ett tydligt beslutsramverk etablerat
- ha en första extern version (hemsida eller pitch)

---

Detta dokument är levande.
Det ska justeras när omdömet växer – inte när trender skiftar.
---
I'll analyze your vision document in connection with your homelab to create actionable steps that leverage your practical experience.

Analysis: Connecting Vision to Homelab

Your homelab is an ideal training arena (Strategic Track 1) because it contains:
- Real Type 1 decisions (GitOps, Redpanda over Kafka, Rust over Node.js)
- Integration complexity (IoT → MQTT → Kafka → DB → API → SPA)
- Evidence of simplification (removed Redpanda operator, rpk-based approach)
- Authentication architecture (Authentik + OIDC)

  ---
Prioritized Actions

Within 30 Days

1. Extract 5 Architecture Principles from Homelab Decisions

Your homelab already embodies implicit principles. Make them explicit:
┌───────────────────────────────────────────────────────┬────────────────────────────────────────────────┐
│                  Candidate Principle                  │             Evidence from Homelab              │
├───────────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ "Prefer operational simplicity over feature richness" │ Removed Redpanda operator, use rpk Job instead │
├───────────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ "Make decisions reversible by default"                │ GitOps enables rollback, sealed secrets        │
├───────────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ "Separate concerns at infrastructure boundaries"      │ Distinct namespaces, clear service boundaries  │
├───────────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ "Own your data pipeline end-to-end"                   │ IoT → MQTT → Redpanda → TimescaleDB flow       │
├───────────────────────────────────────────────────────┼────────────────────────────────────────────────┤
│ "Automate recovery, not just deployment"              │ FluxCD reconciliation, backup CronJobs         │
└───────────────────────────────────────────────────────┴────────────────────────────────────────────────┘
Pros:
- Directly fulfills 30-day goal
- Based on real decisions, not theory
- Creates talking points for clients

Cons:
- May be too specific to homelab context
- Needs validation in enterprise settings

Action: Create docs/ARCHITECTURE_PRINCIPLES.md documenting each principle with homelab evidence.

  ---
2. Write 3-5 ADRs for Key Type 1 Decisions

Document decisions that were "hard to reverse":
┌──────────────────────────────────────────┬─────────────────────┬────────────────────────────────────┐
│                 Decision                 │        Type         │             ADR Value              │
├──────────────────────────────────────────┼─────────────────────┼────────────────────────────────────┤
│ GitOps with FluxCD over ArgoCD or manual │ Type 1              │ Shows evaluation criteria          │
├──────────────────────────────────────────┼─────────────────────┼────────────────────────────────────┤
│ Redpanda over Kafka                      │ Type 1              │ Cost/complexity trade-off          │
├──────────────────────────────────────────┼─────────────────────┼────────────────────────────────────┤
│ rpk-based topic management over operator │ Type 1→2 conversion │ Shows simplification skill         │
├──────────────────────────────────────────┼─────────────────────┼────────────────────────────────────┤
│ Rust/Axum over Node.js for APIs          │ Type 1              │ Performance vs ecosystem trade-off │
├──────────────────────────────────────────┼─────────────────────┼────────────────────────────────────┤
│ TimescaleDB over InfluxDB                │ Type 1              │ SQL compatibility reasoning        │
└──────────────────────────────────────────┴─────────────────────┴────────────────────────────────────┘
Pros:
- Creates portfolio of decision-making
- Demonstrates "decision framework" in action
- Forces articulation of trade-offs
- Reusable template for client work

Cons:
- Retrospective documentation is harder than real-time
- Time investment for past decisions

Action: Create docs/adr/ directory with template and first 3 ADRs.

  ---
3. Document Integration Pattern as Reference

Your data pipeline is a complete integration reference:

IoT Device → MQTT (Mosquitto) → Redpanda → TimescaleDB
↓
homelab-api (REST)
↓
heatpump-web (SPA)

Pros:
- Shows end-to-end system thinking
- Demonstrates event-driven architecture
- Directly applicable to enterprise IoT/telemetry
- Visual content for website

Cons:
- Scale differences from enterprise
- May need abstraction layer

Action: Create docs/REFERENCE_ARCHITECTURE.md with diagrams and decision rationale.

  ---
Within 90 Days

4. Create Case Studies from Homelab Decisions

Package specific decisions as narratives:
┌──────────────────────────────────────┬─────────────────────────────────────────────────────┐
│              Case Study              │                    Narrative Arc                    │
├──────────────────────────────────────┼─────────────────────────────────────────────────────┤
│ "Removing the Redpanda Operator"     │ Complexity → diagnosis → simplification → outcome   │
├──────────────────────────────────────┼─────────────────────────────────────────────────────┤
│ "Choosing Rust for Backend Services" │ Requirements → evaluation → trade-offs → validation │
├──────────────────────────────────────┼─────────────────────────────────────────────────────┤
│ "GitOps Journey"                     │ Manual → semi-automated → full GitOps → lessons     │
└──────────────────────────────────────┴─────────────────────────────────────────────────────┘
Pros:
- Demonstrates "before/after" improvement
- Shows practical problem-solving
- Content for website (Strategic Track 3)
- Evidence of "reducing technical debt"

Cons:
- Requires storytelling skill
- Homelab scale may seem trivial

Mitigation: Focus on decision process, not scale.

Action: Write 2-3 case studies in docs/case-studies/ format.

  ---
5. Build Decision Framework Documentation

Formalize how you categorize decisions:

Type 1 (Homelab examples):
- Platform choice (k3s over k8s)
- Database selection (TimescaleDB)
- GitOps tool (FluxCD)

Type 2 (Homelab examples):
- CI/CD details (BuildKit, crane tagging)
- Library choices (Axum, TanStack Query)
- Retention periods for topics

Pros:
- Fulfills 90-day goal (established framework)
- Creates reusable client artifact
- Shows systematic thinking

Cons:
- May feel academic without real client context

Action: Create docs/DECISION_FRAMEWORK.md with homelab examples.

  ---
6. Document Failure Cases and Learnings

Your recent work had real failures worth documenting:
┌────────────────────────────────┬──────────────────────────────────────────────┐
│            Failure             │                   Learning                   │
├────────────────────────────────┼──────────────────────────────────────────────┤
│ GLIBC 2.38 mismatch            │ Runtime image must match build dependencies  │
├────────────────────────────────┼──────────────────────────────────────────────┤
│ Redpanda operator CRD issues   │ Simpler solutions often more robust          │
├────────────────────────────────┼──────────────────────────────────────────────┤
│ BuildKit multi-tag limitations │ Separate tools for separate concerns (crane) │
├────────────────────────────────┼──────────────────────────────────────────────┤
│ Axum route syntax change       │ Keep dependencies updated, read changelogs   │
└────────────────────────────────┴──────────────────────────────────────────────┘
Pros:
- Demonstrates learning mindset
- Authentic content (not polished marketing)
- Shows real troubleshooting skill
- Builds trust with clients

Cons:
- Exposes mistakes
- May feel uncomfortable

Verdict: Authenticity builds trust. Clients value architects who learn.

Action: Create docs/LESSONS_LEARNED.md or blog-style posts.

  ---
7. Create External-Facing Portfolio Piece

Transform homelab documentation into client-ready content:
┌────────────────────────┬─────────────────────────────────────────┐
│        Internal        │            External Version             │
├────────────────────────┼─────────────────────────────────────────┤
│ CLAUDE.md              │ "How I approach homelab architecture"   │
├────────────────────────┼─────────────────────────────────────────┤
│ ADRs                   │ "Decision documentation template"       │
├────────────────────────┼─────────────────────────────────────────┤
│ Reference architecture │ "Event-driven IoT architecture pattern" │
├────────────────────────┼─────────────────────────────────────────┤
│ Case studies           │ Blog posts or LinkedIn articles         │
└────────────────────────┴─────────────────────────────────────────┘
Pros:
- Fulfills Strategic Track 3 (website)
- Shows thinking, not just doing
- Attracts "rätt personer" as you stated

Cons:
- Requires writing/editing effort
- Public exposure

Action: Select 2-3 pieces to polish for external publication.

  ---
Ongoing Actions

8. Continue ADRs for New Decisions

As you evolve the homelab, document:
- Why you add/remove components
- Trade-offs considered
- Alternatives rejected

9. Validate Principles in Current Job

Test homelab-derived principles against enterprise context:
- Do they scale?
- Do they apply?
- What adjustments needed?

10. Build "Technology Radar"

Optional but valuable:
┌────────┬───────────────────────────────────────────────┐
│  Ring  │                 Technologies                  │
├────────┼───────────────────────────────────────────────┤
│ Adopt  │ k3s, FluxCD, Rust/Axum, Redpanda, TimescaleDB │
├────────┼───────────────────────────────────────────────┤
│ Trial  │ Backstage/TechDocs                            │
├────────┼───────────────────────────────────────────────┤
│ Assess │ (future candidates)                           │
├────────┼───────────────────────────────────────────────┤
│ Hold   │ Redpanda Operator, k3d for production         │
└────────┴───────────────────────────────────────────────┘
  ---
Summary: Alignment with Vision
┌────────────────────────────────────┬────────────────────────────────────────────────────┐
│            Vision Goal             │                   Homelab Action                   │
├────────────────────────────────────┼────────────────────────────────────────────────────┤
│ "Sätta teknisk riktning"           │ Document principles, create reference architecture │
├────────────────────────────────────┼────────────────────────────────────────────────────┤
│ "Minska framtida risk"             │ ADRs show risk-aware decisions                     │
├────────────────────────────────────┼────────────────────────────────────────────────────┤
│ "Göra team självständiga"          │ Documentation enables others to understand/operate │
├────────────────────────────────────┼────────────────────────────────────────────────────┤
│ "Träna och bevisa metod"           │ Case studies demonstrate method in action          │
├────────────────────────────────────┼────────────────────────────────────────────────────┤
│ "Hemsida som visar hur jag tänker" │ External portfolio from homelab content            │
└────────────────────────────────────┴────────────────────────────────────────────────────┘
  ---
Recommended Sequence

Week 1-2:  Extract 5 principles → docs/ARCHITECTURE_PRINCIPLES.md
Week 2-3:  Write first 3 ADRs → docs/adr/
Week 3-4:  Document reference architecture → docs/REFERENCE_ARCHITECTURE.md
Week 5-8:  Write 2 case studies
Week 8-12: Decision framework + external content

This approach uses your homelab as evidence for the vision: "Den här personen hjälper oss att tänka klarare."