# Körschema-modul — spec för integration i befintlig bokningssajt

> **Till Claude Code:** Detta dokument beskriver en ny modul, "Körschema", som ska byggas in i min
> befintliga webbapp (ett bokningssystem med inloggning). Undersök först projektets befintliga stack,
> auth-lösning och datalager och **återanvänd dem** — inför inga nya ramverk, ORM:er eller UI-bibliotek
> utan att fråga. Allt innehåll (faser, pass, övningar, diskussionsämnen) finns som komplett JSON-seed
> längst ner i dokumentet och ska läsas in som seed-data, inte hårdkodas i komponenter.

## 1. Syfte

Mina söner övningskör privat inför B-körkortsprovet. Modulen är en digital version av vårt
lektionsschema: 30 körpass i 6 faser, där varje pass har ett mål, avbockningsbara övningar,
diskussionsämnen ("förstå varför") och handledartips. Eleven ska kunna följa sin progress,
förbereda sig inför nästa pass och blicka tillbaka. Jag (admin/handledare) bockar av övningar.

## 2. Roller och rättigheter

| Förmåga | Elev (inloggad) | Admin (jag) |
|---|---|---|
| Se hela schemat (alla faser/pass/övningar/diskussionsämnen) | ✅ | ✅ |
| Se sin egen progress (procent, avklarade pass, nästa pass) | ✅ endast egen | ✅ alla elever |
| Bocka av / avbocka övningar | ❌ (read-only) | ✅ per elev |
| Skriva passanteckning (kort reflektion efter passet) | ✅ egen | ✅ |
| Se andra elevers progress | ❌ | ✅ |

- Det finns två elever (mina söner) med **helt separat progress**. Modellen ska dock klara N elever.
- Koppla eleverna till befintliga användarkonton i bokningssystemet om sådana finns; annars skapa
  en enkel `students`-koppling till user-tabellen.
- Avbockning är **endast admin** — det är en medveten pedagogisk poäng (handledaren bedömer när ett
  moment sitter). Bygg dock rättighetskollen så att den är en enradsändring att lätta på senare.

## 3. Datamodell (anpassa namn/konventioner till befintlig kodbas)

Statiskt innehåll (från seed-JSON, avsnitt 7):

- `phases`: id, ordning, namn, meta (tidsangivelse), intro
- `lessons`: id, phase_id, ordning (1–30), titel, undertitel, mål (`goal`), handledartips (`tip`)
- `exercises`: id, lesson_id, ordning, text
- `discussion_topics`: id, lesson_id, ordning, text
- `milestones`: id, placering (efter fas X), titel, text (t.ex. "Boka Riskettan")

Dynamisk data:

- `exercise_checks`: student_id, exercise_id, checked_by (admin-user), checked_at
- `lesson_notes`: student_id, lesson_id, författare (elev/admin), text, created_at

Härledda värden (beräkna, lagra inte): progress-% = avbockade övningar / totalt antal övningar;
ett pass är "klart" när alla dess övningar är avbockade; "nästa pass" = första ej klara passet i ordning.

## 4. Vyer

### 4.1 Elevvy — "Mitt körschema"
- **Toppsektion:** progressbar med procent, antal klara pass (X/30), och kort "Nästa pass"-kort
  med passets titel, mål och diskussionsämnen (så att eleven kan **förbereda sig** — läsa på
  diskussionsfrågorna innan passet).
- **Faslista:** 6 faser som sektioner, pass som expanderbara kort (nummer, titel, undertitel,
  klart-markering ✓). Expanderat kort visar: mål, övningar med bockstatus (read-only för elev),
  diskussionsämnen, ev. anteckningar. Handledartipsen (`tip`) visas **endast för admin** — de är
  riktade till mig, inte eleven.
- **Blicka bakåt:** avklarade pass förblir öppningsbara med datum för avbockningar och anteckningar.
- Milstolpe-kort (Riskettan, Risk 2 + kunskapsprov) visas mellan rätt faser för båda rollerna.
- Statiska infosektioner (visas för båda roller, från seed-JSON `staticSections`):
  "Innan ni börjar" (formella krav) och "Det här bedömer förarprövaren" (bedömningsområdena).

### 4.2 Adminvy
- Elevväljare (flikar/dropdown) → samma schemavy men med **klickbara checkboxar** per övning.
- Avbockning sparas direkt (optimistisk UI ok) med tidsstämpel.
- Översikt: per elev — progress-%, senast avbockade pass, nästa pass.
- Möjlighet att skriva anteckning på ett pass.

### 4.3 Integration med bokningssystemet (valfritt, om enkelt)
- Om bokningssystemet har bokningsbara tider: visa "Nästa pass"-förslaget i anslutning till
  bokningsflödet, så att en bokad tid kan taggas med ett pass-nummer. Bygg detta bara om det går
  att göra utan att röra bokningslogikens kärna.

## 5. UI-riktlinjer

- Följ sajtens befintliga designsystem/komponenter. Referensdesignen (fristående HTML-prototyp)
  använder: mörk teal-header, kortbaserad layout, expanderbara pass-kort med nummerbricka som blir
  grön ✓ när passet är klart, gula "milstolpe"-kort, och diskussionsämnen i en ljusgul ruta med 💬.
  Efterlikna känslan men med sajtens egna färger/typografi.
- Mobilanpassat — schemat kommer ofta öppnas i mobil, i eller vid bilen.
- Svenska i hela UI:t.

## 6. Acceptanskriterier

1. Elev ser hela schemat men kan inte ändra någon bockstatus (verifierat även på API-nivå, inte bara dolt i UI).
2. Admin kan växla mellan elever och bocka av/avbocka; ändringen syns för eleven vid nästa laddning.
3. Progress beräknas korrekt: 120 övningar totalt; pass markeras ✓ först när alla dess övningar är avbockade.
4. "Nästa pass" pekar alltid på första ej klara passet och visar dess mål + diskussionsämnen.
5. Seed-datan nedan läses in via seed/migration-script; innehållet finns inte duplicerat i komponentkod.
6. Befintlig bokningsfunktionalitet är oförändrad (regressionstesta inloggning + bokning).

## 7. Innehållsdata (seed)

> Komplett innehåll. `tip` är handledartext (visas endast för admin). Alla texter är på svenska
> och ska bevaras exakt.

```json
{
  "staticSections": {
    "beforeYouStart": {
      "title": "Det formella — checklista för privat övningskörning",
      "requirements": [
        {"title": "Körkortstillstånd", "text": "Eleven ansöker hos Transportstyrelsen (hälsodeklaration + synintyg). Krävs innan första passet."},
        {"title": "Introduktionsutbildning", "text": "Både handledare och elev måste ha gått den (giltig i 5 år). Görs på trafikskola."},
        {"title": "Godkänd handledare", "text": "Handledaren ansöker om handledarskap hos Transportstyrelsen — ett godkännande per elev."},
        {"title": "Grön ÖVNINGSKÖR-skylt", "text": "Bak på bilen vid varje pass. Kontrollera också att bilens försäkring gäller vid övningskörning."}
      ],
      "beforeExam": [
        "Riskutbildning del 1 (\"Riskettan\" — alkohol, droger, trötthet, riskbeteende). Gör den tidigt, gärna runt Fas 3 — den ger stoff till diskussionerna.",
        "Riskutbildning del 2 (halkbanan). Kräver god körvana — lägg den i Fas 6, nära provet. Båda gäller i 5 år.",
        "Kunskapsprovet görs först; godkänt kunskapsprov har begränsad giltighetstid (för närvarande 4 månader), så boka körprovet inom den tiden. Plugga teori parallellt med körningen hela vägen."
      ]
    },
    "assessment": {
      "title": "Det här bedömer förarprövaren på körprovet",
      "intro": "Provet tar 45 minuter: säkerhetskontroll, eventuell särskild manövrering och körning i blandad trafik. Prövaren gör en helhetsbedömning — enstaka småfel fäller inte, men brister i något av dessa områden gör det:",
      "areas": [
        {"title": "Hastighetsanpassning", "text": "Rätt fart för situationen — inte bara under gränsen, utan anpassad till sikt, väglag och omgivning."},
        {"title": "Placering", "text": "Rätt körfält och rätt placering i körfältet, särskilt inför sväng."},
        {"title": "Avsökning & riskmedvetenhet", "text": "Blicken långt fram, speglar, döda vinkeln — och att eleven visar att hen ser riskerna."},
        {"title": "Samspel & flyt", "text": "Tydlig, beslutsam körning som andra trafikanter kan läsa. Överdriven tvekan kan fälla."},
        {"title": "Regeltillämpning", "text": "Väjningsregler, skyltar, vägmarkeringar — tillämpade i praktiken, inte bara i teorin."},
        {"title": "Sparsam körning", "text": "Planerad körning med motorbroms och jämn fart bedöms också — mjuk körning är säker körning."}
      ],
      "outro": "I provet ingår ofta självständig körning: att köra mot ett mål efter skyltar eller enkel vägbeskrivning. Det tränas särskilt i Fas 4–6."
    }
  },
  "milestones": [
    {"afterPhase": 3, "icon": "⚠️", "title": "Boka nu: Riskutbildning del 1 (\"Riskettan\")", "text": "Obligatorisk före provet och som bäst just här — teorin om alkohol, trötthet och grupptryck ger djup åt diskussionerna i Fas 4–6."},
    {"afterPhase": 5, "icon": "🧊", "title": "Boka nu: Riskutbildning del 2 (halkbanan) + kunskapsprovet", "text": "Risk 2 kräver god körvana — perfekt läge i början av Fas 6. Kunskapsprovet ska vara godkänt innan körprovet kan göras, och giltighetstiden är begränsad — planera ihop dem."}
  ],
  "phases": [
    {
      "order": 1,
      "name": "Grunderna — manövrering",
      "meta": "Pass 1–5 · Stor tom parkering eller övningsplats · ca vecka 1–2",
      "intro": "Allt sker i låg fart på en säker, tom yta. Målet är att bilens reglage ska bli automatiska så att all uppmärksamhet senare kan läggas på trafiken. Stressa inte vidare — den som är trygg här lär sig resten dubbelt så fort.",
      "lessons": [
        {
          "order": 1,
          "title": "Bekanta dig med bilen",
          "subtitle": "Körställning, reglage, första metrarna",
          "goal": "Eleven kan göra i ordning körställningen själv och flytta bilen kontrollerat framåt och stanna mjukt.",
          "exercises": [
            "Ställ in stol, ratt, huvudstöd, speglar och bälte — helt på egen hand",
            "Gå igenom reglage: blinkers, torkare, ljus, varningsblinkers, handbroms",
            "Starta och stäng av motorn, hitta kopplingens dragläge (manuell växellåda)",
            "Krypkör 20–30 meter och stanna mjukt på utpekad punkt — upprepa många gånger"
          ],
          "discussionTopics": [
            "Varför är körställningen en säkerhetsfråga och inte bara bekvämlighet? (kontroll över reglagen, sikt, och krockskydd — huvudstödet skyddar nacken)",
            "Vad gör kopplingen egentligen i bilen? Att förstå det gör dragläget logiskt i stället för magiskt",
            "Prata om era roller: du är lärare och lagkamrat, inte domare. Vad ska eleven säga om det känns för svårt?"
          ],
          "tip": "Sitt själv i förarsätet först och visa lugnt. Kör sedan inte mer än 45 min — första passet är mentalt tröttande."
        },
        {
          "order": 2,
          "title": "Krypkörning och broms",
          "subtitle": "Dragläge, mjuk broms, hård broms",
          "goal": "Eleven kan krypköra i kontrollerad fart och bromsa både mjukt till punkt och hårt utan att tveka.",
          "exercises": [
            "Krypkör i åtta-figur och slalom mellan koner/flaskor",
            "Stanna mjukt exakt vid en linje — sista metern i krypfart",
            "Hård inbromsning från ca 30 km/h — våga trycka ordentligt",
            "Backa rakt 10–20 meter med blicken bakåt genom rutan"
          ],
          "discussionTopics": [
            "Dubbla farten ger fyrdubbel bromssträcka — räkna tillsammans: hur långt rullar bilen på 1 sekunds reaktionstid i 30, 50, 90 km/h?",
            "Varför övar vi hård broms redan nu? (På provet ingår ofta effektiv bromsning från 50 km/h — och i verkligheten får man inte tveka)",
            "Varför tittar man bakåt genom rutan när man backar, och inte bara i speglarna?"
          ],
          "tip": "Många nybörjare bromsar för försiktigt av rädsla att 'göra fel'. Beröm en rejäl inbromsning."
        },
        {
          "order": 3,
          "title": "Växling",
          "subtitle": "Uppväxling, nedväxling, motorbroms",
          "goal": "Eleven växlar 1–3 utan att titta ner och förstår sambandet mellan växel, varvtal och fart.",
          "exercises": [
            "Uppväxling 1→2→3 med blicken framåt hela tiden",
            "Nedväxling och stanna, starta igen — repetera tills det flyter",
            "Sakta in med motorbroms i stället för fotbroms",
            "Kombinationsbana: starta, växla upp, slalom, bromsa, backa"
          ],
          "discussionTopics": [
            "Vad säger motorljudet dig? Lär eleven växla på örat i stället för att stirra på varvräknaren",
            "Varför bedöms 'sparsam körning' på körprovet? (planering, motorbroms och jämn fart = både miljö och säkerhet — den som planerar hinner se risker)",
            "Automat? Diskutera skillnaden och vad ett automat-villkor på körkortet innebär"
          ],
          "tip": "Kör ni automat: lägg tiden på krypkörning med enbart broms och på blickteknik i stället."
        },
        {
          "order": 4,
          "title": "Backning och start i lutning",
          "subtitle": "Backa mot mål, backstart utan rullning",
          "goal": "Eleven backar kontrollerat mot ett mål och startar i uppförslutning utan att rulla bakåt.",
          "exercises": [
            "Backa rakt längs en linje, sedan backa i kurva mot ett mål",
            "Backstart i uppförsbacke med parkeringsbroms",
            "Start i nedförslutning — hålla emot med broms",
            "Rulla aldrig: träna att alltid säkra bilen med broms/handbroms vid stopp i lutning"
          ],
          "discussionTopics": [
            "Vilka är de typiska backningsolyckorna? (barn och låga hinder bakom bilen — därför: alltid en långsam fart och full uppsikt, aldrig chansning)",
            "Varför ingår start i lutning ofta i körprovet? Vad visar momentet prövaren? (fordonskontroll under press)",
            "Vad gör du om bilen börjar rulla åt fel håll — panik eller plan?"
          ],
          "tip": "Hitta en lagom brant backe utan trafik. Låt eleven misslyckas några gånger — det avdramatiserar."
        },
        {
          "order": 5,
          "title": "Styrteknik och kombination",
          "subtitle": "Rattteknik, blicken, allt i ett",
          "goal": "Eleven styr med korrekt ratteknik och klarar en hel 'bana' med start, växling, slalom, vändpunkt, backning och parkering i ruta.",
          "exercises": [
            "Ratta med växelvis grepp ('mata ratten') i åtta-figur",
            "Kör en hel kombinationsbana ni bygger ihop — ta tid, gör om, förbättra",
            "Kör samma bana åt andra hållet",
            "Parkera i ruta framlänges och baklänges (grovt — finlir kommer i Fas 2)"
          ],
          "discussionTopics": [
            "'Blicken styr' — varför hamnar bilen där man tittar? Testa: titta på konen = köra på konen",
            "Vad av det ni tränat känns automatiskt nu, och vad kräver fortfarande tankekraft? (det som kräver tankekraft är inte redo för trafik än)",
            "Sätt ett gemensamt mål: vad ska sitta innan vi ger oss ut i trafik?"
          ],
          "tip": "Milstolpe! När detta pass sitter är eleven redo för nästa fas. Fira med något."
        }
      ]
    },
    {
      "order": 2,
      "name": "Manövrering och parkering",
      "meta": "Pass 6–9 · Lugna gator och parkeringar · ca vecka 2–3",
      "intro": "Nu flyttar ni till verkliga men lugna miljöer. Parkering och vändning är klassiska provmoment — men viktigare: de tränar precision, uppsikt och tålamod.",
      "lessons": [
        {
          "order": 6,
          "title": "Vändning",
          "subtitle": "Trepunktsvändning, U-sväng, välja plats",
          "goal": "Eleven kan vända på smal gata på flera sätt och — viktigast — välja en säker plats och metod själv.",
          "exercises": [
            "Trepunktsvändning på smal gata med full uppsikt åt alla håll",
            "U-sväng där gatan är bred nog",
            "Vända genom att backa in i gathörn/utfart",
            "Övning: 'vänd på lämpligt ställe' — eleven väljer själv plats och metod"
          ],
          "discussionTopics": [
            "På provet säger prövaren ofta bara 'vänd på lämplig plats' — vad gör en plats lämplig eller olämplig? (sikt, trafik, backkrön, utfarter)",
            "Varför är metodvalet en del av bedömningen? (omdöme väger tyngre än teknik)",
            "Vad är farligast vid vändning — och var ska blicken vara i varje delmoment?"
          ],
          "tip": "Säg 'vänd på lämplig plats' precis som prövaren gör, och låt eleven resonera högt."
        },
        {
          "order": 7,
          "title": "Fickparkering",
          "subtitle": "Parkera längs kant, mellan bilar",
          "goal": "Eleven fickparkerar med referenspunkter, rättar till vid behov, och lämnar fickan säkert.",
          "exercises": [
            "Fickparkering bakom en ensam bil — hitta era referenspunkter",
            "Fickparkering mellan två bilar (börja med stora luckor)",
            "Köra ut ur fickan: blinkers, döda vinkeln, vänta på lucka",
            "Parkera på höger respektive vänster sida (enkelriktat)"
          ],
          "discussionTopics": [
            "Vad händer om parkeringen blir sned på provet? (Att lugnt ta om är helt okej — prövaren bedömer säkerhet och uppsikt, inte elegans)",
            "Varför är utfarten ur fickan riskablare än infarten? (cyklister!)",
            "Referenspunkter: varför fungerar de, och varför måste var och en hitta sina egna?"
          ],
          "tip": "Ställ upp egna 'bilar' med koner/soptunnor först om riktiga bilar känns för nervöst."
        },
        {
          "order": 8,
          "title": "Vinkelparkering och backning runt hörn",
          "subtitle": "P-rutor, backa runt gathörn, lutning",
          "goal": "Eleven parkerar i ruta framåt/bakåt, backar runt gathörn med korrekt uppsikt och säkrar bilen i lutning.",
          "exercises": [
            "Vinkelparkering framlänges och baklänges mellan bilar",
            "Backa runt gathörn — långsam fart, uppsikt runt om, väja för trafik",
            "Parkera i lutning: rätt rattställning, handbroms, växel i",
            "Träna på trång parkeringsplats (t.ex. matbutik en lugn tid)"
          ],
          "discussionTopics": [
            "Varför backar man in i p-rutan? (utsikten när man ska ut igen — säkerheten flyttas till det kontrollerade momentet)",
            "Åt vilket håll vinklar man hjulen i uppförs- respektive nedförslutning, och varför?",
            "Vem bär ansvaret om det blir en skada på parkeringsplats? Vad gör man?"
          ],
          "tip": "Backning runt hörn avslöjar direkt om uppsikten brister — perfekt moment att träna 'titta först, agera sen'."
        },
        {
          "order": 9,
          "title": "Säkerhetskontroll",
          "subtitle": "Provet börjar innan bilen rullar",
          "goal": "Eleven kan självständigt genomföra den säkerhetskontroll som inleder körprovet.",
          "exercises": [
            "Kontrollera all belysning, blinkers och reflexer — en person trycker, en tittar",
            "Kontrollera däck: mönsterdjup (minst 1,6 mm), lufttryck, skador",
            "Kontrollera vätskor: spolarvätska, kylarvätska, olja, bromsvätska",
            "Bromsprov och styrservokontroll, torkare/spolare, varningslampor i instrumentpanelen — gör hela kontrollen som ett 'prov' med dig som prövare"
          ],
          "discussionTopics": [
            "Vem är juridiskt ansvarig för att bilen är i trafikdugligt skick — ägaren eller föraren? (föraren!)",
            "Vilka fel gör bilen direkt olaglig att köra? Vad gör man om en lampa är trasig på väg någonstans?",
            "Varför inleder Trafikverket provet med just detta moment?"
          ],
          "tip": "Använd er egen bil och gör kontrollen till rutin — låt eleven göra en snabbversion före varje pass framöver."
        }
      ]
    },
    {
      "order": 3,
      "name": "Trafik i lugn tätort",
      "meta": "Pass 10–14 · Villaområden, mindre gator · ca vecka 3–5",
      "intro": "Första riktiga trafiken. Välj lugna villaområden med 30/40-gränser. Nu flyttas fokus från bilen till omgivningen: avsökning, väjningsregler och oskyddade trafikanter. Boka gärna Riskettan under den här fasen.",
      "lessons": [
        {
          "order": 10,
          "title": "Första turen i trafik",
          "subtitle": "Avsökning, fartanpassning i 30/40",
          "goal": "Eleven kör lugnt i villaområde med blicken långt fram och rätt fart för miljön.",
          "exercises": [
            "Kör 20–30 min i lugnt villaområde, du navigerar i god tid",
            "Träna avsökning: blicken långt fram, sidogator, utfarter — be eleven berätta högt vad hen ser",
            "Fartanpassning: 30 vid skola/lekplats, krypfart vid skymd sikt",
            "Spegelrutin: back- och sidospeglar var 5–10 sekund"
          ],
          "discussionTopics": [
            "Vad betyder 'defensiv körning'? (att köra så att andras misstag inte blir olyckor — förvänta dig fel av andra)",
            "Varför är rätt fart mer än att hålla hastighetsgränsen? (gränsen är ett tak, inte ett riktvärde — sikt och miljö styr)",
            "'Kommentera-körning': låt eleven prata högt om allt hen ser och planerar. Varför avslöjar det avsökningen?"
          ],
          "tip": "Kommentera-körning (eleven berättar högt) är ditt bästa verktyg hela vägen till provet — du ser exakt vad eleven ser och missar."
        },
        {
          "order": 11,
          "title": "Väjningsregler i praktiken",
          "subtitle": "Högerregeln, väjningsplikt, stopplikt, utfart",
          "goal": "Eleven identifierar vilken regel som gäller i varje korsning och agerar tydligt och rätt.",
          "exercises": [
            "Planera en runda som blandar högerregelkorsningar, väjningsplikt och stopplikt",
            "Stopplikt: helt stopp, rätt plats, ordentlig avsökning åt båda håll",
            "Utfartsregeln: över gångbana, från parkering, bensinmack",
            "Be eleven säga högt vid varje korsning: 'här gäller... för att...'"
          ],
          "discussionTopics": [
            "Varför ska man aldrig 'vinka fram' andra eller köra på någon annans framvinkning? (regler skapar förutsägbarhet — vänlighet skapar olyckor)",
            "Skillnaden mellan väjningsplikt och stopplikt — varför finns stopplikt på vissa platser?",
            "Vad kommunicerar din fart till andra? (den som rullar in mjukt mot korsningen 'lovar' att stanna)"
          ],
          "tip": "Högerregelkorsningar utan skyltar i villaområden är det som oftast överraskar — nöt dem."
        },
        {
          "order": 12,
          "title": "Korsningar och svängar",
          "subtitle": "Placering, blinkers, körfältsval",
          "goal": "Eleven placerar bilen rätt före sväng, blinkar i rätt tid och genomför svängar med korrekt uppsikt.",
          "exercises": [
            "Högersväng: placering nära kant, kolla cykelbana, snäv sväng",
            "Vänstersväng: placering mot mitten, väja för mötande, rätt körfält efteråt",
            "Blinkers-timing: i god tid men inte vilseledande tidigt",
            "Träna komplexa korsningar med flera körfält och refuger"
          ],
          "discussionTopics": [
            "Varför är placeringen före svängen ett språk? (den berättar för alla andra vad du tänker göra — fel placering ljuger)",
            "Bristande avsökning i korsning är en av de vanligaste orsakerna till underkänt körprov — vad tror ni prövaren letar efter med blicken?",
            "Vad gör du om du hamnat i fel körfält? (kör fel och rätta säkert — aldrig tvärbyta)"
          ],
          "tip": "Filma gärna (passageraren!) en tur och titta tillsammans efteråt — placeringen syns tydligt på film."
        },
        {
          "order": 13,
          "title": "Oskyddade trafikanter",
          "subtitle": "Övergångsställen, cyklister, dolda risker",
          "goal": "Eleven visar tydligt sänkt fart och beredskap vid alla platser där oskyddade trafikanter kan dyka upp.",
          "exercises": [
            "Planera rutt förbi skola, övergångsställen, cykelpassager och cykelöverfarter",
            "Träna: sänk farten tidigt och synligt inför övergångsställe, sök ögonkontakt",
            "Passera cyklister med god marginal, vänta bakom vid behov",
            "'Dolda hindret': skymda utfarter, bussar vid hållplats, parkerade bilar — vad kan finnas bakom?"
          ],
          "discussionTopics": [
            "Krockvåld: vid 30 km/h överlever nästan alla fotgängare en påkörning, vid 50 dör många. Vad betyder det för valet av fart där människor rör sig?",
            "Skillnaden mellan cykelpassage och cykelöverfart — vem väjer för vem?",
            "Varför bedömer prövaren detta stenhårt? (det är här körkortets verkliga ansvar ligger — liv)"
          ],
          "tip": "Detta är provets viktigaste tema. En förare som är snabb i övrigt men slarvig här blir underkänd — med rätta."
        },
        {
          "order": 14,
          "title": "Repetition och första självständigheten",
          "subtitle": "Eleven planerar och kör — du är tyst",
          "goal": "Eleven genomför en hel runda i lugn tätort i princip utan stöd, och kan utvärdera sig själv efteråt.",
          "exercises": [
            "Eleven planerar rutten själv (ca 30 min, ska innehålla korsningstyper, parkering, vändning)",
            "Du är tyst hela rundan utom vid säkerhetsrisk",
            "Avsluta med en fickparkering och en vändning på lämplig plats",
            "Gemensam utvärdering: eleven först — vad gick bra, vad behöver mer?"
          ],
          "discussionTopics": [
            "Hur kändes det att köra utan stöd? Vilka beslut var svårast?",
            "På provet ingår ofta självständig körning mot mål — varför testar Trafikverket förmågan att köra utan instruktioner?",
            "Är eleven redo för stadstrafik? Bestäm tillsammans — och lyssna på magkänslan."
          ],
          "tip": "Milstolpe! Om detta pass går bra: vidare till Fas 4. Om inte: repetera valda delar — det är helt normalt att den här fasen tar flest pass."
        }
      ]
    },
    {
      "order": 4,
      "name": "Stadstrafik",
      "meta": "Pass 15–19 · Stadskärna, tätare trafik · ca vecka 5–7",
      "intro": "Högre tempo, fler intryck, fler samspelssituationer. Börja utanför rusningstid och öka svårighetsgraden. Nu ska teorin sitta parallellt — boka kunskapsprov mot slutet av denna fas eller nästa.",
      "lessons": [
        {
          "order": 15,
          "title": "Cirkulationsplatser",
          "subtitle": "Små och stora rondeller, körfältsval",
          "goal": "Eleven kör genom cirkulationsplatser med rätt körfält, bra flyt och korrekt blinkers.",
          "exercises": [
            "Liten enfilig rondell: väjning vid infart, blinkers vid utfart",
            "Flerfilig cirkulationsplats: körfältsval före infart, hålla fältet igenom",
            "Kör samma rondell till alla utfarter, flera varv träning",
            "Samspel: ta lucka utan att tveka för länge, utan att chansa"
          ],
          "discussionTopics": [
            "Varför blinkar man ut ur rondellen men (oftast) inte in? Vad hjälper det de som väntar?",
            "Rondellens idé: varför är cirkulationsplatser säkrare än korsningar trots att de känns rörigare? (lägre fart, färre konfliktpunkter, inga frontalkrockar)",
            "Vad gör du om du är i fel körfält inne i rondellen? (kör ett varv till — aldrig tvärbyta)"
          ],
          "tip": "Hitta en lugn rondell och kör den 10 gånger i rad. Tråkigt men extremt effektivt."
        },
        {
          "order": 16,
          "title": "Körfältsbyten och tät trafik",
          "subtitle": "Spegel–tecken–döda vinkeln",
          "goal": "Eleven byter körfält säkert med fullständig rutin och håller flyt i tätare trafik.",
          "exercises": [
            "Rutin: spegel → blinkers → döda vinkeln → mjukt byte — nöt tills det är reflex",
            "Byten i tätare trafik: bedöma luckor, anpassa fart till luckan",
            "Kollektivkörfält och bussgator: se skyltarna, håll dig rätt",
            "Väja för buss som lämnar hållplats (tätort)"
          ],
          "discussionTopics": [
            "Ställ er vid bilen och visa fysiskt hur stor döda vinkeln är — låt eleven sitta i förarsätet medan du 'försvinner' i den. Varför räcker inte speglarna?",
            "Varför kan överdriven tvekan fällas på provet? (oförutsägbarhet är en risk — trafik bygger på att andra kan läsa dig)",
            "Hur hittar man luckan — anpassar man sin fart till luckan eller väntar på en perfekt lucka?"
          ],
          "tip": "Om eleven glömmer döda vinkeln: säg ingenting förrän efter passet, räkna missarna, visa siffran. Effektivt."
        },
        {
          "order": 17,
          "title": "Vägmarkeringar, trafikljus och enkelriktat",
          "subtitle": "Läsa staden",
          "goal": "Eleven läser och följer stadens 'text': markeringar, ljus, skyltar, enkelriktade gator.",
          "exercises": [
            "Rutt genom område med enkelriktade gator — eleven navigerar efter skyltarna",
            "Spärrlinjer, körfältspilar, stopplinjer — följ dem exakt",
            "Trafikljus: träna beslutet vid gult ('hinner jag stanna säkert?')",
            "Vändning och parkering på enkelriktad gata"
          ],
          "discussionTopics": [
            "Gult ljus: vad säger regeln egentligen — och var går din personliga 'beslutslinje' i olika farter?",
            "Varför är körfältspilar juridiskt bindande? Vad gör du om du står i svängfält men ska rakt fram?",
            "Hur 'läser' man en okänd stadsmiljö snabbt? (markeringar och skyltar berättar allt — om man tittar)"
          ],
          "tip": "Låt eleven köra i ett okänt område — hemmagator kan man utantill, provet sker delvis på 'nya' gator."
        },
        {
          "order": 18,
          "title": "Rusningstrafik",
          "subtitle": "Tempo, tålamod och beslut under press",
          "goal": "Eleven behåller rutiner, omdöme och lugn även i tät och stressig trafik.",
          "exercises": [
            "Kör i eftermiddagsrusning (start: utkanten av rusningen)",
            "Trängda situationer: blockerade korsningar (kör aldrig in om du inte kommer ur), bilköer, otåliga medtrafikanter",
            "Håll rutinerna under press: speglar, avstånd, avsökning",
            "Avsluta i lugnare miljö och varva ner"
          ],
          "discussionTopics": [
            "Vad händer med ditt omdöme när någon ligger tätt bakom och stressar? Hur kopplar man bort det? (släpp förbi, håll din plan)",
            "Grupptryck i bilen: kompisar som hetsar, musik, mobil — hur säger man nej? (detta är Riskettans kärna — knyt ihop!)",
            "Varför är trött/stressad/arg farligare än oerfaren?"
          ],
          "tip": "Kör detta pass bara när grunderna sitter. En dålig upplevelse i rusning kan knäcka självförtroendet — dosera."
        },
        {
          "order": 19,
          "title": "Stadskörning mot mål",
          "subtitle": "Självständig körning på riktigt",
          "goal": "Eleven kör självständigt mot mål efter skyltar, gör säkra vägval och hanterar felval lugnt.",
          "exercises": [
            "'Kör mot Centrum, följ sedan skyltar mot X' — inga fler instruktioner",
            "Träna att medvetet missa en avfart/sväng och lösa det säkert",
            "Hela paletten: rondell, ljus, körfältsbyten, oskyddade trafikanter — du bedömer tyst",
            "Utvärdering med provets ögon: hastighet, placering, avsökning, samspel, regler, sparsamhet"
          ],
          "discussionTopics": [
            "Hellre missa målet än göra en farlig manöver — varför bedömer prövaren ett 'säkert fel vägval' som godkänt men en tvärnit vid avfarten som underkänt?",
            "Hur navigerar man efter skyltar utan att tappa avsökningen? (planera i god tid, läs skyltarna tidigt)",
            "Milstolpe-fråga: vad skiljer nu elevens körning från en färdig förares?"
          ],
          "tip": "Milstolpe! Klarar eleven detta lugnt är stadsdelen av provet inom räckhåll. Nästa fas: högre farter."
        }
      ]
    },
    {
      "order": 5,
      "name": "Landsväg och motorväg",
      "meta": "Pass 20–24 · 70–110 km/h · ca vecka 7–9",
      "intro": "Höga farter kräver planering på längre avstånd. Landsvägen är statistiskt farligast — här sker de allvarligaste olyckorna. Perfekta pass för de stora 'varför'-samtalen.",
      "lessons": [
        {
          "order": 20,
          "title": "Landsväg — fart och avstånd",
          "subtitle": "70/80/90, avstånd, kurvteknik",
          "goal": "Eleven håller rätt fart och säkert avstånd på landsväg och anpassar farten till kurvor och sikt.",
          "exercises": [
            "Kör 70/80/90-vägar, träna att snabbt komma upp i rätt fart",
            "Tresekundersregeln: räkna avstånd till framförvarande, öka vid regn/mörker",
            "Kurvteknik: bromsa före, jämn fart igenom, accelerera ut",
            "Backkrön och skymd sikt: vad kan finnas bakom? Fartanpassning"
          ],
          "discussionTopics": [
            "Varför dör flest i trafiken på just landsväg? (hög fart + möten utan mitträcke + omkörningar = störst krockvåld)",
            "Tresekundersregeln: varför avstånd i tid och inte meter? Räkna: 3 sekunder i 90 km/h = 75 meter",
            "Mötande trafik på smal väg — vad är din plan om någon kommer över på din sida?"
          ],
          "tip": "Att våga köra i 90 är ett steg för många nybörjare — för låg fart på landsväg är faktiskt också ett provfel."
        },
        {
          "order": 21,
          "title": "Anslutningar och vänstersväng på landsväg",
          "subtitle": "Av- och påfarter, det farligaste momentet",
          "goal": "Eleven hanterar anslutningar till landsväg och genomför vänstersväng över mötande trafik med god marginal.",
          "exercises": [
            "Sväng in på landsväg från mindre väg: bedöma luckor i hög fart",
            "Vänstersväng från landsväg: blinkers tidigt, ligg rätt placerad, vänta med raka hjul",
            "Högersväng från landsväg: sakta in i god tid, blinkers tidigt (tänk på bakomvarande i 90!)",
            "Busshållplatser och långsamma fordon: passera med omdöme"
          ],
          "discussionTopics": [
            "Varför räknas vänstersväng på landsväg till de farligaste momenten i trafiken? (stillastående mitt i 90-trafik + korsar mötande)",
            "Varför raka hjul medan man väntar på att svänga vänster? (blir du påkörd bakifrån knuffas du rakt fram — inte in i mötande)",
            "Hur bedömer man luckor när mötande kommer i 90? (avstånd i hög fart bedöms nästan alltid för optimistiskt)"
          ],
          "tip": "Öva luckbedömning som passagerare först: 'hade du svängt nu?' — facit kommer några sekunder senare."
        },
        {
          "order": 22,
          "title": "Omkörning och järnvägskorsning",
          "subtitle": "Beslutet, genomförandet — och att avstå",
          "goal": "Eleven kan genomföra en säker omkörning, bli omkörd, och passera järnvägskorsningar korrekt.",
          "exercises": [
            "Kör om ett långsamt fordon (traktor/lastbil) där sikten är god: avstånd, sikt, beslut, snabbt genomförande",
            "Träna att AVSTÅ: påbörja bedömning, avbryt när något talar emot",
            "Bli omkörd: håll fart och kant, underlätta",
            "Järnvägskorsningar med och utan bommar: sakta in, titta, aldrig stanna på spåret"
          ],
          "discussionTopics": [
            "Räkna på tidsvinsten: att köra om någon som kör 80 i stället för 90 sparar under en mil bara någon minut — vad väger vinsten mot risken?",
            "Varför är 'avstå' det vanligaste rätta svaret på omkörningsfrågan — och varför är det ett styrkebesked, inte feghet?",
            "Varför chansar man aldrig vid järnväg, ens när det 'ser tomt ut'? (ett tåg kan inte väja och behöver upp till en kilometer för att stanna)"
          ],
          "tip": "Finns ingen naturlig omkörningssituation — träna momentet mentalt: 'skulle du köra om nu? varför/varför inte?'"
        },
        {
          "order": 23,
          "title": "Motorväg",
          "subtitle": "Påfart, körfältsdisciplin, avfart",
          "goal": "Eleven kör på motorväg med rätt påfartsteknik, god körfältsdisciplin och säker avfart.",
          "exercises": [
            "Påfart: accelerera på accelerationsfältet till trafikens rytm, hitta luckan med spegel + döda vinkeln",
            "Ligg i höger körfält, kör om vänster, tillbaka höger",
            "Håll avstånd i 110 — räkna tre sekunder",
            "Avfart: blinkers tidigt, bromsa PÅ avfarten (inte på motorvägen), läs av hastighetsskyltar"
          ],
          "discussionTopics": [
            "Varför ska man vara uppe i trafikens fart redan vid påfartens slut? (fartskillnad är motorvägens största risk — inte farten i sig)",
            "Varför är motorvägen statistiskt vår säkraste vägtyp trots högst fart? (inga möten, inga korsningar — knyt tillbaka till landsvägssamtalet)",
            "Trötthet på långkörning: vilka är varningssignalerna och vad är enda botemedlet? (paus/sömn — inte kaffe, inte hög musik)"
          ],
          "tip": "Välj en lugn tid för första motorvägspasset. Påfarten är det svåra — kör gärna av och på flera gånger i rad."
        },
        {
          "order": 24,
          "title": "Mörker och svåra förhållanden",
          "subtitle": "Mörkerkörning, regn, det ni inte kan välja",
          "goal": "Eleven behärskar ljusanvändning i mörker och anpassar körningen till nedsatt sikt och sämre väggrepp.",
          "exercises": [
            "Mörkerkörning (kör sena kvällspass när mörkret kommit): hel-/halvljusteknik, bländning vid möte, upptäcka gående/reflexer",
            "Möte i mörker på landsväg: blända av i rätt ögonblick, blicken höger om mötande ljus",
            "Regnkörning: längre avstånd, aquaplaning-risk, torkare/ljus",
            "Diskutera halka teoretiskt inför Risk 2 — ni kan inte träna det säkert själva"
          ],
          "discussionTopics": [
            "I mörker ser du bara så långt strålkastarna når — hur fort får du då köra egentligen? (aldrig fortare än att du kan stanna inom den sträcka du ser)",
            "En gående utan reflex syns på ~25 m med halvljus, med reflex på ~125 m — vad betyder det i 90 km/h?",
            "Varför finns Risk 2/halkbanan? Vad tror eleven halka känns som — och boka nu: Risk 2 ska göras i nästa fas"
          ],
          "tip": "I augusti–september blir det mörkt sent — lägg mörkerpasset sent på kvällen eller skjut det framåt i schemat. Momentet är obligatoriskt att kunna, även om provet körs i dagsljus."
        }
      ]
    },
    {
      "order": 6,
      "name": "Provträning",
      "meta": "Pass 25–30 · Blandade miljöer, gärna provorten · ca vecka 9–11",
      "intro": "Nu tränar ni som det testas. Risk 2 (halkbanan) görs i denna fas, kunskapsprovet ska vara klart eller nära, och körprovet bokas. Din roll skiftar från lärare till tyst bedömare.",
      "lessons": [
        {
          "order": 25,
          "title": "Körning med prövarens ögon",
          "subtitle": "Genomgång av bedömningen + testrunda",
          "goal": "Eleven vet exakt vad som bedöms på provet och kan koppla varje bedömningsområde till sin egen körning.",
          "exercises": [
            "Gå igenom bedömningsområdena i rutan högst upp — tillsammans, punkt för punkt",
            "Kör 30 min blandad trafik där eleven själv säger till när hen gjort något prövaren skulle notera",
            "Träna säkerhetskontrollen igen — nu på tid, självständigt",
            "Lista elevens 2–3 svagaste områden — de styr nästa pass"
          ],
          "discussionTopics": [
            "Prövaren gör en helhetsbedömning — varför fäller inte en stannad motor eller en omtagen parkering? Vad fäller? (mönster av brister, farliga situationer, brist på avsökning)",
            "Vad tror eleven är sina svaga punkter? Stämmer det med din bild? (självinsikt bedöms indirekt — den säkra föraren känner sina brister)",
            "Hur ser en godkänd körning ut — perfekt eller trygg?"
          ],
          "tip": "Från och med nu: ge instruktioner exakt som prövaren — 'vid nästa korsning, sväng vänster', i god tid, lugnt."
        },
        {
          "order": 26,
          "title": "Provsimulering 1",
          "subtitle": "Hela provet, på riktigt, med dig som prövare",
          "goal": "Eleven genomför ett komplett simulerat körprov: säkerhetskontroll, manövrering och 35–40 min körning.",
          "exercises": [
            "Säkerhetskontroll på tid, utan hjälp",
            "Manövermoment: effektiv bromsning från 50, backning eller vändning, parkering",
            "35–40 min körning: tätort + landsväg/motorväg, inkl. självständig körning mot mål",
            "Du är helt tyst utom instruktioner. Anteckna diskret. Utvärdera EFTERÅT — eleven självvärderar först"
          ],
          "discussionTopics": [
            "Gå igenom rundan mot bedömningsområdena ett i taget: var stod det starkt, var brast det?",
            "Hur påverkade 'provkänslan' körningen? Vad av det försvinner med rutin, vad behöver hanteras?",
            "Bestäm tillsammans: vad ska pass 27 fokusera på?"
          ],
          "tip": "Gör simuleringen på en tid och i miljöer som liknar det riktiga provet. Boka gärna det riktiga körprovet nu — ett datum skärper träningen."
        },
        {
          "order": 27,
          "title": "Träna svagheterna",
          "subtitle": "Riktad träning på det som brast",
          "goal": "De 2–3 svagaste områdena från simuleringen är märkbart förbättrade.",
          "exercises": [
            "Bygg hela passet kring svagheterna från Provsimulering 1 (t.ex. 10 rondeller i rad, eller enbart vänstersvängar)",
            "Repetera det svåraste manövermomentet tills det sitter",
            "Kommentera-körning på just de svaga momenten",
            "Avsluta med 10 min felfri, lugn körning — sluta på topp"
          ],
          "discussionTopics": [
            "Varför är det effektivare att nöta ett moment tio gånger än att köra tio blandade rundor? (repetition i tät följd bygger automatik)",
            "Har svagheten en gemensam rot? (många ytfel — t.ex. sen blinkers, missad spegel — bottnar i samma sak: för sen planering)",
            "Hur vet man själv att ett moment 'sitter'? (när det går bra även en dålig dag)"
          ],
          "tip": "Om Risk 2 inte är gjord — senast nu! Den kräver god körvana och ska vara giltig på provdagen, precis som Risk 1."
        },
        {
          "order": 28,
          "title": "Provsimulering 2",
          "subtitle": "Generalrepetition med allt",
          "goal": "Ett fullständigt simulerat prov som eleven klarar på 'godkänd-nivå' enligt er gemensamma bedömning.",
          "exercises": [
            "Full simulering igen: säkerhetskontroll, manöver, 40 min med självständig körning",
            "Nya vägar — inte samma runda som Simulering 1",
            "Kasta in en överraskning: 'vänd på lämplig plats', oplanerad parkering",
            "Bedöm tillsammans mot alla områden: godkänt eller inte? Var ärliga"
          ],
          "discussionTopics": [
            "Vad skiljer denna körning från Simulering 1? Låt eleven sätta ord på sin egen utveckling",
            "Om detta hade varit provet — godkänt? Om tvekan: vad exakt saknas, och hur många pass behövs? (att skjuta provet en vecka är billigare än ett omprov)",
            "Hur hanterar man ett misstag MITT i provet? (släpp det direkt — prövaren bedömer helheten, många klarar provet trots ett dåligt moment)"
          ],
          "tip": "Är ni tveksamma — kör en extra vecka. Elever som övningskört mycket privat OCH tagit någon enstaka lektion på trafikskola har högst godkännandegrad; en 'kontrollektion' med trafiklärare nu är en klok investering."
        },
        {
          "order": 29,
          "title": "Finslipning och sparsam körning",
          "subtitle": "Flyt, mjukhet, marginaler",
          "goal": "Körningen är inte bara säker utan mjuk, planerad och sparsam — det som lyfter helhetsintrycket.",
          "exercises": [
            "Fokuspass på planering: släpp gasen tidigt mot rött, motorbroms, jämn fart, glid fram till hinder",
            "Mjuka accelerationer, tidig uppväxling",
            "Perfekta rutiner: speglar, blinkers, avsökning — inga missar på hela passet",
            "Kort teorirepetition: skyltar och regler som känns osäkra (kunskapsprovet ska vara klart nu)"
          ],
          "discussionTopics": [
            "Varför hänger sparsam körning och säker körning ihop? (båda handlar om samma sak: att läsa trafiken tidigt och planera)",
            "Nervositet inför provet: vad är elevens plan? (sömn, mat, komma i tid, våga be prövaren upprepa en instruktion — det är helt tillåtet)",
            "Prövaren VILL godkänna — hur förändrar den tanken känslan inför provet?"
          ],
          "tip": "Boka provets praktiska detaljer nu: giltig legitimation, vilken bil ni använder (trafikskolans hyrbil eller egen godkänd bil — kolla kraven på trafikverket.se), tider."
        },
        {
          "order": 30,
          "title": "Generalrepetition i provorten",
          "subtitle": "Sista passet före provet",
          "goal": "Eleven är varm i kläderna i just de miljöer där provet körs och går in i provdagen lugn och förberedd.",
          "exercises": [
            "Kör i området runt Trafikverkets förarprovskontor — de miljötyper som finns där (rondeller? motorvägspåfart? stadskärna?)",
            "En sista lugn helrunda — ingen ny inlärning, bara bekräftelse",
            "Snabb säkerhetskontroll som rutinkoll",
            "Gå igenom provdagen praktiskt: tid, legitimation, bil, vad som händer steg för steg"
          ],
          "discussionTopics": [
            "Vad är planen om det INTE blir godkänt? (helt odramatiskt: man får ett tydligt protokoll på vad som brast, tränar det och bokar om — många duktiga förare klarar det på andra försöket)",
            "Och efter godkänt: de första två åren har nya förare högst olycksrisk — vilka egna regler sätter eleven upp? (inga passagerare som stressar, aldrig trött, aldrig mobil)",
            "Sista frågan, den viktigaste: varför finns egentligen allt det här — provet, reglerna, träningen? (inte för att klara ett prov — för att alla ska komma hem)"
          ],
          "tip": "Dagen före provet: kör INTE ett långt hårt pass. En kort lugn runda eller vila. Lycka till — ni har gjort jobbet! 🎉"
        }
      ]
    }
  ]
}
```

