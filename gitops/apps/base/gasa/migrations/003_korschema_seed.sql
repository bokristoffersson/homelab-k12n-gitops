-- Migration: 003_korschema_seed
-- Description: Seed content for the korschema module (phases, lessons,
-- exercises, discussion topics, milestones, static info sections).
--
-- GENERATED from the JSON block in docs/korschema-spec.md - do not edit
-- the row values by hand; regenerate instead. Re-runs on every Flux sync:
-- upserts on the natural keys (ord / lesson_id+ord) keep the database in
-- sync with this file while the generated PKs - which the dynamic tables
-- (checks, notes) reference - stay stable.
--
-- NOTE: records itself in schema_migrations as (3, 'korschema_seed') at
-- the END of this file, per project convention.

-- Fas 1: Grunderna — manövrering
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (1, 'Grunderna — manövrering', 'Pass 1–5 · Stor tom parkering eller övningsplats · ca vecka 1–2', 'Allt sker i låg fart på en säker, tom yta. Målet är att bilens reglage ska bli automatiska så att all uppmärksamhet senare kan läggas på trafiken. Stressa inte vidare — den som är trygg här lär sig resten dubbelt så fort.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 1), 1, 'Bekanta dig med bilen', 'Körställning, reglage, första metrarna', 'Eleven kan göra i ordning körställningen själv och flytta bilen kontrollerat framåt och stanna mjukt.', 'Sitt själv i förarsätet först och visa lugnt. Kör sedan inte mer än 45 min — första passet är mentalt tröttande.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 1, 'Ställ in stol, ratt, huvudstöd, speglar och bälte — helt på egen hand'),
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 2, 'Gå igenom reglage: blinkers, torkare, ljus, varningsblinkers, handbroms'),
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 3, 'Starta och stäng av motorn, hitta kopplingens dragläge (manuell växellåda)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 4, 'Krypkör 20–30 meter och stanna mjukt på utpekad punkt — upprepa många gånger')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 1, 'Varför är körställningen en säkerhetsfråga och inte bara bekvämlighet? (kontroll över reglagen, sikt, och krockskydd — huvudstödet skyddar nacken)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 2, 'Vad gör kopplingen egentligen i bilen? Att förstå det gör dragläget logiskt i stället för magiskt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 1), 3, 'Prata om era roller: du är lärare och lagkamrat, inte domare. Vad ska eleven säga om det känns för svårt?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 1), 2, 'Krypkörning och broms', 'Dragläge, mjuk broms, hård broms', 'Eleven kan krypköra i kontrollerad fart och bromsa både mjukt till punkt och hårt utan att tveka.', 'Många nybörjare bromsar för försiktigt av rädsla att ''göra fel''. Beröm en rejäl inbromsning.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 1, 'Krypkör i åtta-figur och slalom mellan koner/flaskor'),
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 2, 'Stanna mjukt exakt vid en linje — sista metern i krypfart'),
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 3, 'Hård inbromsning från ca 30 km/h — våga trycka ordentligt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 4, 'Backa rakt 10–20 meter med blicken bakåt genom rutan')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 1, 'Dubbla farten ger fyrdubbel bromssträcka — räkna tillsammans: hur långt rullar bilen på 1 sekunds reaktionstid i 30, 50, 90 km/h?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 2, 'Varför övar vi hård broms redan nu? (På provet ingår ofta effektiv bromsning från 50 km/h — och i verkligheten får man inte tveka)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 2), 3, 'Varför tittar man bakåt genom rutan när man backar, och inte bara i speglarna?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 1), 3, 'Växling', 'Uppväxling, nedväxling, motorbroms', 'Eleven växlar 1–3 utan att titta ner och förstår sambandet mellan växel, varvtal och fart.', 'Kör ni automat: lägg tiden på krypkörning med enbart broms och på blickteknik i stället.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 1, 'Uppväxling 1→2→3 med blicken framåt hela tiden'),
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 2, 'Nedväxling och stanna, starta igen — repetera tills det flyter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 3, 'Sakta in med motorbroms i stället för fotbroms'),
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 4, 'Kombinationsbana: starta, växla upp, slalom, bromsa, backa')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 1, 'Vad säger motorljudet dig? Lär eleven växla på örat i stället för att stirra på varvräknaren'),
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 2, 'Varför bedöms ''sparsam körning'' på körprovet? (planering, motorbroms och jämn fart = både miljö och säkerhet — den som planerar hinner se risker)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 3), 3, 'Automat? Diskutera skillnaden och vad ett automat-villkor på körkortet innebär')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 1), 4, 'Backning och start i lutning', 'Backa mot mål, backstart utan rullning', 'Eleven backar kontrollerat mot ett mål och startar i uppförslutning utan att rulla bakåt.', 'Hitta en lagom brant backe utan trafik. Låt eleven misslyckas några gånger — det avdramatiserar.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 1, 'Backa rakt längs en linje, sedan backa i kurva mot ett mål'),
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 2, 'Backstart i uppförsbacke med parkeringsbroms'),
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 3, 'Start i nedförslutning — hålla emot med broms'),
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 4, 'Rulla aldrig: träna att alltid säkra bilen med broms/handbroms vid stopp i lutning')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 1, 'Vilka är de typiska backningsolyckorna? (barn och låga hinder bakom bilen — därför: alltid en långsam fart och full uppsikt, aldrig chansning)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 2, 'Varför ingår start i lutning ofta i körprovet? Vad visar momentet prövaren? (fordonskontroll under press)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 4), 3, 'Vad gör du om bilen börjar rulla åt fel håll — panik eller plan?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 1), 5, 'Styrteknik och kombination', 'Rattteknik, blicken, allt i ett', 'Eleven styr med korrekt ratteknik och klarar en hel ''bana'' med start, växling, slalom, vändpunkt, backning och parkering i ruta.', 'Milstolpe! När detta pass sitter är eleven redo för nästa fas. Fira med något.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 1, 'Ratta med växelvis grepp (''mata ratten'') i åtta-figur'),
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 2, 'Kör en hel kombinationsbana ni bygger ihop — ta tid, gör om, förbättra'),
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 3, 'Kör samma bana åt andra hållet'),
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 4, 'Parkera i ruta framlänges och baklänges (grovt — finlir kommer i Fas 2)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 1, '''Blicken styr'' — varför hamnar bilen där man tittar? Testa: titta på konen = köra på konen'),
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 2, 'Vad av det ni tränat känns automatiskt nu, och vad kräver fortfarande tankekraft? (det som kräver tankekraft är inte redo för trafik än)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 5), 3, 'Sätt ett gemensamt mål: vad ska sitta innan vi ger oss ut i trafik?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Fas 2: Manövrering och parkering
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (2, 'Manövrering och parkering', 'Pass 6–9 · Lugna gator och parkeringar · ca vecka 2–3', 'Nu flyttar ni till verkliga men lugna miljöer. Parkering och vändning är klassiska provmoment — men viktigare: de tränar precision, uppsikt och tålamod.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 2), 6, 'Vändning', 'Trepunktsvändning, U-sväng, välja plats', 'Eleven kan vända på smal gata på flera sätt och — viktigast — välja en säker plats och metod själv.', 'Säg ''vänd på lämplig plats'' precis som prövaren gör, och låt eleven resonera högt.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 1, 'Trepunktsvändning på smal gata med full uppsikt åt alla håll'),
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 2, 'U-sväng där gatan är bred nog'),
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 3, 'Vända genom att backa in i gathörn/utfart'),
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 4, 'Övning: ''vänd på lämpligt ställe'' — eleven väljer själv plats och metod')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 1, 'På provet säger prövaren ofta bara ''vänd på lämplig plats'' — vad gör en plats lämplig eller olämplig? (sikt, trafik, backkrön, utfarter)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 2, 'Varför är metodvalet en del av bedömningen? (omdöme väger tyngre än teknik)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 6), 3, 'Vad är farligast vid vändning — och var ska blicken vara i varje delmoment?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 2), 7, 'Fickparkering', 'Parkera längs kant, mellan bilar', 'Eleven fickparkerar med referenspunkter, rättar till vid behov, och lämnar fickan säkert.', 'Ställ upp egna ''bilar'' med koner/soptunnor först om riktiga bilar känns för nervöst.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 1, 'Fickparkering bakom en ensam bil — hitta era referenspunkter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 2, 'Fickparkering mellan två bilar (börja med stora luckor)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 3, 'Köra ut ur fickan: blinkers, döda vinkeln, vänta på lucka'),
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 4, 'Parkera på höger respektive vänster sida (enkelriktat)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 1, 'Vad händer om parkeringen blir sned på provet? (Att lugnt ta om är helt okej — prövaren bedömer säkerhet och uppsikt, inte elegans)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 2, 'Varför är utfarten ur fickan riskablare än infarten? (cyklister!)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 7), 3, 'Referenspunkter: varför fungerar de, och varför måste var och en hitta sina egna?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 2), 8, 'Vinkelparkering och backning runt hörn', 'P-rutor, backa runt gathörn, lutning', 'Eleven parkerar i ruta framåt/bakåt, backar runt gathörn med korrekt uppsikt och säkrar bilen i lutning.', 'Backning runt hörn avslöjar direkt om uppsikten brister — perfekt moment att träna ''titta först, agera sen''.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 1, 'Vinkelparkering framlänges och baklänges mellan bilar'),
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 2, 'Backa runt gathörn — långsam fart, uppsikt runt om, väja för trafik'),
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 3, 'Parkera i lutning: rätt rattställning, handbroms, växel i'),
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 4, 'Träna på trång parkeringsplats (t.ex. matbutik en lugn tid)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 1, 'Varför backar man in i p-rutan? (utsikten när man ska ut igen — säkerheten flyttas till det kontrollerade momentet)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 2, 'Åt vilket håll vinklar man hjulen i uppförs- respektive nedförslutning, och varför?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 8), 3, 'Vem bär ansvaret om det blir en skada på parkeringsplats? Vad gör man?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 2), 9, 'Säkerhetskontroll', 'Provet börjar innan bilen rullar', 'Eleven kan självständigt genomföra den säkerhetskontroll som inleder körprovet.', 'Använd er egen bil och gör kontrollen till rutin — låt eleven göra en snabbversion före varje pass framöver.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 1, 'Kontrollera all belysning, blinkers och reflexer — en person trycker, en tittar'),
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 2, 'Kontrollera däck: mönsterdjup (minst 1,6 mm), lufttryck, skador'),
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 3, 'Kontrollera vätskor: spolarvätska, kylarvätska, olja, bromsvätska'),
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 4, 'Bromsprov och styrservokontroll, torkare/spolare, varningslampor i instrumentpanelen — gör hela kontrollen som ett ''prov'' med dig som prövare')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 1, 'Vem är juridiskt ansvarig för att bilen är i trafikdugligt skick — ägaren eller föraren? (föraren!)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 2, 'Vilka fel gör bilen direkt olaglig att köra? Vad gör man om en lampa är trasig på väg någonstans?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 9), 3, 'Varför inleder Trafikverket provet med just detta moment?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Fas 3: Trafik i lugn tätort
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (3, 'Trafik i lugn tätort', 'Pass 10–14 · Villaområden, mindre gator · ca vecka 3–5', 'Första riktiga trafiken. Välj lugna villaområden med 30/40-gränser. Nu flyttas fokus från bilen till omgivningen: avsökning, väjningsregler och oskyddade trafikanter. Boka gärna Riskettan under den här fasen.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 3), 10, 'Första turen i trafik', 'Avsökning, fartanpassning i 30/40', 'Eleven kör lugnt i villaområde med blicken långt fram och rätt fart för miljön.', 'Kommentera-körning (eleven berättar högt) är ditt bästa verktyg hela vägen till provet — du ser exakt vad eleven ser och missar.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 1, 'Kör 20–30 min i lugnt villaområde, du navigerar i god tid'),
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 2, 'Träna avsökning: blicken långt fram, sidogator, utfarter — be eleven berätta högt vad hen ser'),
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 3, 'Fartanpassning: 30 vid skola/lekplats, krypfart vid skymd sikt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 4, 'Spegelrutin: back- och sidospeglar var 5–10 sekund')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 1, 'Vad betyder ''defensiv körning''? (att köra så att andras misstag inte blir olyckor — förvänta dig fel av andra)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 2, 'Varför är rätt fart mer än att hålla hastighetsgränsen? (gränsen är ett tak, inte ett riktvärde — sikt och miljö styr)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 10), 3, '''Kommentera-körning'': låt eleven prata högt om allt hen ser och planerar. Varför avslöjar det avsökningen?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 3), 11, 'Väjningsregler i praktiken', 'Högerregeln, väjningsplikt, stopplikt, utfart', 'Eleven identifierar vilken regel som gäller i varje korsning och agerar tydligt och rätt.', 'Högerregelkorsningar utan skyltar i villaområden är det som oftast överraskar — nöt dem.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 1, 'Planera en runda som blandar högerregelkorsningar, väjningsplikt och stopplikt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 2, 'Stopplikt: helt stopp, rätt plats, ordentlig avsökning åt båda håll'),
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 3, 'Utfartsregeln: över gångbana, från parkering, bensinmack'),
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 4, 'Be eleven säga högt vid varje korsning: ''här gäller... för att...''')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 1, 'Varför ska man aldrig ''vinka fram'' andra eller köra på någon annans framvinkning? (regler skapar förutsägbarhet — vänlighet skapar olyckor)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 2, 'Skillnaden mellan väjningsplikt och stopplikt — varför finns stopplikt på vissa platser?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 11), 3, 'Vad kommunicerar din fart till andra? (den som rullar in mjukt mot korsningen ''lovar'' att stanna)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 3), 12, 'Korsningar och svängar', 'Placering, blinkers, körfältsval', 'Eleven placerar bilen rätt före sväng, blinkar i rätt tid och genomför svängar med korrekt uppsikt.', 'Filma gärna (passageraren!) en tur och titta tillsammans efteråt — placeringen syns tydligt på film.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 1, 'Högersväng: placering nära kant, kolla cykelbana, snäv sväng'),
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 2, 'Vänstersväng: placering mot mitten, väja för mötande, rätt körfält efteråt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 3, 'Blinkers-timing: i god tid men inte vilseledande tidigt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 4, 'Träna komplexa korsningar med flera körfält och refuger')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 1, 'Varför är placeringen före svängen ett språk? (den berättar för alla andra vad du tänker göra — fel placering ljuger)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 2, 'Bristande avsökning i korsning är en av de vanligaste orsakerna till underkänt körprov — vad tror ni prövaren letar efter med blicken?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 12), 3, 'Vad gör du om du hamnat i fel körfält? (kör fel och rätta säkert — aldrig tvärbyta)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 3), 13, 'Oskyddade trafikanter', 'Övergångsställen, cyklister, dolda risker', 'Eleven visar tydligt sänkt fart och beredskap vid alla platser där oskyddade trafikanter kan dyka upp.', 'Detta är provets viktigaste tema. En förare som är snabb i övrigt men slarvig här blir underkänd — med rätta.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 1, 'Planera rutt förbi skola, övergångsställen, cykelpassager och cykelöverfarter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 2, 'Träna: sänk farten tidigt och synligt inför övergångsställe, sök ögonkontakt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 3, 'Passera cyklister med god marginal, vänta bakom vid behov'),
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 4, '''Dolda hindret'': skymda utfarter, bussar vid hållplats, parkerade bilar — vad kan finnas bakom?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 1, 'Krockvåld: vid 30 km/h överlever nästan alla fotgängare en påkörning, vid 50 dör många. Vad betyder det för valet av fart där människor rör sig?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 2, 'Skillnaden mellan cykelpassage och cykelöverfart — vem väjer för vem?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 13), 3, 'Varför bedömer prövaren detta stenhårt? (det är här körkortets verkliga ansvar ligger — liv)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 3), 14, 'Repetition och första självständigheten', 'Eleven planerar och kör — du är tyst', 'Eleven genomför en hel runda i lugn tätort i princip utan stöd, och kan utvärdera sig själv efteråt.', 'Milstolpe! Om detta pass går bra: vidare till Fas 4. Om inte: repetera valda delar — det är helt normalt att den här fasen tar flest pass.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 1, 'Eleven planerar rutten själv (ca 30 min, ska innehålla korsningstyper, parkering, vändning)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 2, 'Du är tyst hela rundan utom vid säkerhetsrisk'),
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 3, 'Avsluta med en fickparkering och en vändning på lämplig plats'),
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 4, 'Gemensam utvärdering: eleven först — vad gick bra, vad behöver mer?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 1, 'Hur kändes det att köra utan stöd? Vilka beslut var svårast?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 2, 'På provet ingår ofta självständig körning mot mål — varför testar Trafikverket förmågan att köra utan instruktioner?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 14), 3, 'Är eleven redo för stadstrafik? Bestäm tillsammans — och lyssna på magkänslan.')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Fas 4: Stadstrafik
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (4, 'Stadstrafik', 'Pass 15–19 · Stadskärna, tätare trafik · ca vecka 5–7', 'Högre tempo, fler intryck, fler samspelssituationer. Börja utanför rusningstid och öka svårighetsgraden. Nu ska teorin sitta parallellt — boka kunskapsprov mot slutet av denna fas eller nästa.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 4), 15, 'Cirkulationsplatser', 'Små och stora rondeller, körfältsval', 'Eleven kör genom cirkulationsplatser med rätt körfält, bra flyt och korrekt blinkers.', 'Hitta en lugn rondell och kör den 10 gånger i rad. Tråkigt men extremt effektivt.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 1, 'Liten enfilig rondell: väjning vid infart, blinkers vid utfart'),
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 2, 'Flerfilig cirkulationsplats: körfältsval före infart, hålla fältet igenom'),
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 3, 'Kör samma rondell till alla utfarter, flera varv träning'),
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 4, 'Samspel: ta lucka utan att tveka för länge, utan att chansa')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 1, 'Varför blinkar man ut ur rondellen men (oftast) inte in? Vad hjälper det de som väntar?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 2, 'Rondellens idé: varför är cirkulationsplatser säkrare än korsningar trots att de känns rörigare? (lägre fart, färre konfliktpunkter, inga frontalkrockar)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 15), 3, 'Vad gör du om du är i fel körfält inne i rondellen? (kör ett varv till — aldrig tvärbyta)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 4), 16, 'Körfältsbyten och tät trafik', 'Spegel–tecken–döda vinkeln', 'Eleven byter körfält säkert med fullständig rutin och håller flyt i tätare trafik.', 'Om eleven glömmer döda vinkeln: säg ingenting förrän efter passet, räkna missarna, visa siffran. Effektivt.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 1, 'Rutin: spegel → blinkers → döda vinkeln → mjukt byte — nöt tills det är reflex'),
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 2, 'Byten i tätare trafik: bedöma luckor, anpassa fart till luckan'),
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 3, 'Kollektivkörfält och bussgator: se skyltarna, håll dig rätt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 4, 'Väja för buss som lämnar hållplats (tätort)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 1, 'Ställ er vid bilen och visa fysiskt hur stor döda vinkeln är — låt eleven sitta i förarsätet medan du ''försvinner'' i den. Varför räcker inte speglarna?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 2, 'Varför kan överdriven tvekan fällas på provet? (oförutsägbarhet är en risk — trafik bygger på att andra kan läsa dig)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 16), 3, 'Hur hittar man luckan — anpassar man sin fart till luckan eller väntar på en perfekt lucka?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 4), 17, 'Vägmarkeringar, trafikljus och enkelriktat', 'Läsa staden', 'Eleven läser och följer stadens ''text'': markeringar, ljus, skyltar, enkelriktade gator.', 'Låt eleven köra i ett okänt område — hemmagator kan man utantill, provet sker delvis på ''nya'' gator.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 1, 'Rutt genom område med enkelriktade gator — eleven navigerar efter skyltarna'),
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 2, 'Spärrlinjer, körfältspilar, stopplinjer — följ dem exakt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 3, 'Trafikljus: träna beslutet vid gult (''hinner jag stanna säkert?'')'),
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 4, 'Vändning och parkering på enkelriktad gata')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 1, 'Gult ljus: vad säger regeln egentligen — och var går din personliga ''beslutslinje'' i olika farter?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 2, 'Varför är körfältspilar juridiskt bindande? Vad gör du om du står i svängfält men ska rakt fram?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 17), 3, 'Hur ''läser'' man en okänd stadsmiljö snabbt? (markeringar och skyltar berättar allt — om man tittar)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 4), 18, 'Rusningstrafik', 'Tempo, tålamod och beslut under press', 'Eleven behåller rutiner, omdöme och lugn även i tät och stressig trafik.', 'Kör detta pass bara när grunderna sitter. En dålig upplevelse i rusning kan knäcka självförtroendet — dosera.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 1, 'Kör i eftermiddagsrusning (start: utkanten av rusningen)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 2, 'Trängda situationer: blockerade korsningar (kör aldrig in om du inte kommer ur), bilköer, otåliga medtrafikanter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 3, 'Håll rutinerna under press: speglar, avstånd, avsökning'),
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 4, 'Avsluta i lugnare miljö och varva ner')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 1, 'Vad händer med ditt omdöme när någon ligger tätt bakom och stressar? Hur kopplar man bort det? (släpp förbi, håll din plan)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 2, 'Grupptryck i bilen: kompisar som hetsar, musik, mobil — hur säger man nej? (detta är Riskettans kärna — knyt ihop!)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 18), 3, 'Varför är trött/stressad/arg farligare än oerfaren?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 4), 19, 'Stadskörning mot mål', 'Självständig körning på riktigt', 'Eleven kör självständigt mot mål efter skyltar, gör säkra vägval och hanterar felval lugnt.', 'Milstolpe! Klarar eleven detta lugnt är stadsdelen av provet inom räckhåll. Nästa fas: högre farter.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 1, '''Kör mot Centrum, följ sedan skyltar mot X'' — inga fler instruktioner'),
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 2, 'Träna att medvetet missa en avfart/sväng och lösa det säkert'),
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 3, 'Hela paletten: rondell, ljus, körfältsbyten, oskyddade trafikanter — du bedömer tyst'),
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 4, 'Utvärdering med provets ögon: hastighet, placering, avsökning, samspel, regler, sparsamhet')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 1, 'Hellre missa målet än göra en farlig manöver — varför bedömer prövaren ett ''säkert fel vägval'' som godkänt men en tvärnit vid avfarten som underkänt?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 2, 'Hur navigerar man efter skyltar utan att tappa avsökningen? (planera i god tid, läs skyltarna tidigt)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 19), 3, 'Milstolpe-fråga: vad skiljer nu elevens körning från en färdig förares?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Fas 5: Landsväg och motorväg
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (5, 'Landsväg och motorväg', 'Pass 20–24 · 70–110 km/h · ca vecka 7–9', 'Höga farter kräver planering på längre avstånd. Landsvägen är statistiskt farligast — här sker de allvarligaste olyckorna. Perfekta pass för de stora ''varför''-samtalen.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 5), 20, 'Landsväg — fart och avstånd', '70/80/90, avstånd, kurvteknik', 'Eleven håller rätt fart och säkert avstånd på landsväg och anpassar farten till kurvor och sikt.', 'Att våga köra i 90 är ett steg för många nybörjare — för låg fart på landsväg är faktiskt också ett provfel.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 1, 'Kör 70/80/90-vägar, träna att snabbt komma upp i rätt fart'),
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 2, 'Tresekundersregeln: räkna avstånd till framförvarande, öka vid regn/mörker'),
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 3, 'Kurvteknik: bromsa före, jämn fart igenom, accelerera ut'),
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 4, 'Backkrön och skymd sikt: vad kan finnas bakom? Fartanpassning')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 1, 'Varför dör flest i trafiken på just landsväg? (hög fart + möten utan mitträcke + omkörningar = störst krockvåld)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 2, 'Tresekundersregeln: varför avstånd i tid och inte meter? Räkna: 3 sekunder i 90 km/h = 75 meter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 20), 3, 'Mötande trafik på smal väg — vad är din plan om någon kommer över på din sida?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 5), 21, 'Anslutningar och vänstersväng på landsväg', 'Av- och påfarter, det farligaste momentet', 'Eleven hanterar anslutningar till landsväg och genomför vänstersväng över mötande trafik med god marginal.', 'Öva luckbedömning som passagerare först: ''hade du svängt nu?'' — facit kommer några sekunder senare.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 1, 'Sväng in på landsväg från mindre väg: bedöma luckor i hög fart'),
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 2, 'Vänstersväng från landsväg: blinkers tidigt, ligg rätt placerad, vänta med raka hjul'),
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 3, 'Högersväng från landsväg: sakta in i god tid, blinkers tidigt (tänk på bakomvarande i 90!)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 4, 'Busshållplatser och långsamma fordon: passera med omdöme')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 1, 'Varför räknas vänstersväng på landsväg till de farligaste momenten i trafiken? (stillastående mitt i 90-trafik + korsar mötande)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 2, 'Varför raka hjul medan man väntar på att svänga vänster? (blir du påkörd bakifrån knuffas du rakt fram — inte in i mötande)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 21), 3, 'Hur bedömer man luckor när mötande kommer i 90? (avstånd i hög fart bedöms nästan alltid för optimistiskt)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 5), 22, 'Omkörning och järnvägskorsning', 'Beslutet, genomförandet — och att avstå', 'Eleven kan genomföra en säker omkörning, bli omkörd, och passera järnvägskorsningar korrekt.', 'Finns ingen naturlig omkörningssituation — träna momentet mentalt: ''skulle du köra om nu? varför/varför inte?''')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 1, 'Kör om ett långsamt fordon (traktor/lastbil) där sikten är god: avstånd, sikt, beslut, snabbt genomförande'),
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 2, 'Träna att AVSTÅ: påbörja bedömning, avbryt när något talar emot'),
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 3, 'Bli omkörd: håll fart och kant, underlätta'),
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 4, 'Järnvägskorsningar med och utan bommar: sakta in, titta, aldrig stanna på spåret')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 1, 'Räkna på tidsvinsten: att köra om någon som kör 80 i stället för 90 sparar under en mil bara någon minut — vad väger vinsten mot risken?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 2, 'Varför är ''avstå'' det vanligaste rätta svaret på omkörningsfrågan — och varför är det ett styrkebesked, inte feghet?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 22), 3, 'Varför chansar man aldrig vid järnväg, ens när det ''ser tomt ut''? (ett tåg kan inte väja och behöver upp till en kilometer för att stanna)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 5), 23, 'Motorväg', 'Påfart, körfältsdisciplin, avfart', 'Eleven kör på motorväg med rätt påfartsteknik, god körfältsdisciplin och säker avfart.', 'Välj en lugn tid för första motorvägspasset. Påfarten är det svåra — kör gärna av och på flera gånger i rad.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 1, 'Påfart: accelerera på accelerationsfältet till trafikens rytm, hitta luckan med spegel + döda vinkeln'),
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 2, 'Ligg i höger körfält, kör om vänster, tillbaka höger'),
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 3, 'Håll avstånd i 110 — räkna tre sekunder'),
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 4, 'Avfart: blinkers tidigt, bromsa PÅ avfarten (inte på motorvägen), läs av hastighetsskyltar')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 1, 'Varför ska man vara uppe i trafikens fart redan vid påfartens slut? (fartskillnad är motorvägens största risk — inte farten i sig)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 2, 'Varför är motorvägen statistiskt vår säkraste vägtyp trots högst fart? (inga möten, inga korsningar — knyt tillbaka till landsvägssamtalet)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 23), 3, 'Trötthet på långkörning: vilka är varningssignalerna och vad är enda botemedlet? (paus/sömn — inte kaffe, inte hög musik)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 5), 24, 'Mörker och svåra förhållanden', 'Mörkerkörning, regn, det ni inte kan välja', 'Eleven behärskar ljusanvändning i mörker och anpassar körningen till nedsatt sikt och sämre väggrepp.', 'I augusti–september blir det mörkt sent — lägg mörkerpasset sent på kvällen eller skjut det framåt i schemat. Momentet är obligatoriskt att kunna, även om provet körs i dagsljus.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 1, 'Mörkerkörning (kör sena kvällspass när mörkret kommit): hel-/halvljusteknik, bländning vid möte, upptäcka gående/reflexer'),
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 2, 'Möte i mörker på landsväg: blända av i rätt ögonblick, blicken höger om mötande ljus'),
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 3, 'Regnkörning: längre avstånd, aquaplaning-risk, torkare/ljus'),
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 4, 'Diskutera halka teoretiskt inför Risk 2 — ni kan inte träna det säkert själva')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 1, 'I mörker ser du bara så långt strålkastarna når — hur fort får du då köra egentligen? (aldrig fortare än att du kan stanna inom den sträcka du ser)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 2, 'En gående utan reflex syns på ~25 m med halvljus, med reflex på ~125 m — vad betyder det i 90 km/h?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 24), 3, 'Varför finns Risk 2/halkbanan? Vad tror eleven halka känns som — och boka nu: Risk 2 ska göras i nästa fas')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Fas 6: Provträning
INSERT INTO korschema_phases (ord, name, meta, intro) VALUES
  (6, 'Provträning', 'Pass 25–30 · Blandade miljöer, gärna provorten · ca vecka 9–11', 'Nu tränar ni som det testas. Risk 2 (halkbanan) görs i denna fas, kunskapsprovet ska vara klart eller nära, och körprovet bokas. Din roll skiftar från lärare till tyst bedömare.')
ON CONFLICT (ord) DO UPDATE SET name = EXCLUDED.name, meta = EXCLUDED.meta, intro = EXCLUDED.intro;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 25, 'Körning med prövarens ögon', 'Genomgång av bedömningen + testrunda', 'Eleven vet exakt vad som bedöms på provet och kan koppla varje bedömningsområde till sin egen körning.', 'Från och med nu: ge instruktioner exakt som prövaren — ''vid nästa korsning, sväng vänster'', i god tid, lugnt.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 1, 'Gå igenom bedömningsområdena i rutan högst upp — tillsammans, punkt för punkt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 2, 'Kör 30 min blandad trafik där eleven själv säger till när hen gjort något prövaren skulle notera'),
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 3, 'Träna säkerhetskontrollen igen — nu på tid, självständigt'),
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 4, 'Lista elevens 2–3 svagaste områden — de styr nästa pass')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 1, 'Prövaren gör en helhetsbedömning — varför fäller inte en stannad motor eller en omtagen parkering? Vad fäller? (mönster av brister, farliga situationer, brist på avsökning)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 2, 'Vad tror eleven är sina svaga punkter? Stämmer det med din bild? (självinsikt bedöms indirekt — den säkra föraren känner sina brister)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 25), 3, 'Hur ser en godkänd körning ut — perfekt eller trygg?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 26, 'Provsimulering 1', 'Hela provet, på riktigt, med dig som prövare', 'Eleven genomför ett komplett simulerat körprov: säkerhetskontroll, manövrering och 35–40 min körning.', 'Gör simuleringen på en tid och i miljöer som liknar det riktiga provet. Boka gärna det riktiga körprovet nu — ett datum skärper träningen.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 1, 'Säkerhetskontroll på tid, utan hjälp'),
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 2, 'Manövermoment: effektiv bromsning från 50, backning eller vändning, parkering'),
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 3, '35–40 min körning: tätort + landsväg/motorväg, inkl. självständig körning mot mål'),
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 4, 'Du är helt tyst utom instruktioner. Anteckna diskret. Utvärdera EFTERÅT — eleven självvärderar först')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 1, 'Gå igenom rundan mot bedömningsområdena ett i taget: var stod det starkt, var brast det?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 2, 'Hur påverkade ''provkänslan'' körningen? Vad av det försvinner med rutin, vad behöver hanteras?'),
  ((SELECT id FROM korschema_lessons WHERE ord = 26), 3, 'Bestäm tillsammans: vad ska pass 27 fokusera på?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 27, 'Träna svagheterna', 'Riktad träning på det som brast', 'De 2–3 svagaste områdena från simuleringen är märkbart förbättrade.', 'Om Risk 2 inte är gjord — senast nu! Den kräver god körvana och ska vara giltig på provdagen, precis som Risk 1.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 1, 'Bygg hela passet kring svagheterna från Provsimulering 1 (t.ex. 10 rondeller i rad, eller enbart vänstersvängar)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 2, 'Repetera det svåraste manövermomentet tills det sitter'),
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 3, 'Kommentera-körning på just de svaga momenten'),
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 4, 'Avsluta med 10 min felfri, lugn körning — sluta på topp')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 1, 'Varför är det effektivare att nöta ett moment tio gånger än att köra tio blandade rundor? (repetition i tät följd bygger automatik)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 2, 'Har svagheten en gemensam rot? (många ytfel — t.ex. sen blinkers, missad spegel — bottnar i samma sak: för sen planering)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 27), 3, 'Hur vet man själv att ett moment ''sitter''? (när det går bra även en dålig dag)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 28, 'Provsimulering 2', 'Generalrepetition med allt', 'Ett fullständigt simulerat prov som eleven klarar på ''godkänd-nivå'' enligt er gemensamma bedömning.', 'Är ni tveksamma — kör en extra vecka. Elever som övningskört mycket privat OCH tagit någon enstaka lektion på trafikskola har högst godkännandegrad; en ''kontrollektion'' med trafiklärare nu är en klok investering.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 1, 'Full simulering igen: säkerhetskontroll, manöver, 40 min med självständig körning'),
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 2, 'Nya vägar — inte samma runda som Simulering 1'),
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 3, 'Kasta in en överraskning: ''vänd på lämplig plats'', oplanerad parkering'),
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 4, 'Bedöm tillsammans mot alla områden: godkänt eller inte? Var ärliga')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 1, 'Vad skiljer denna körning från Simulering 1? Låt eleven sätta ord på sin egen utveckling'),
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 2, 'Om detta hade varit provet — godkänt? Om tvekan: vad exakt saknas, och hur många pass behövs? (att skjuta provet en vecka är billigare än ett omprov)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 28), 3, 'Hur hanterar man ett misstag MITT i provet? (släpp det direkt — prövaren bedömer helheten, många klarar provet trots ett dåligt moment)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 29, 'Finslipning och sparsam körning', 'Flyt, mjukhet, marginaler', 'Körningen är inte bara säker utan mjuk, planerad och sparsam — det som lyfter helhetsintrycket.', 'Boka provets praktiska detaljer nu: giltig legitimation, vilken bil ni använder (trafikskolans hyrbil eller egen godkänd bil — kolla kraven på trafikverket.se), tider.')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 1, 'Fokuspass på planering: släpp gasen tidigt mot rött, motorbroms, jämn fart, glid fram till hinder'),
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 2, 'Mjuka accelerationer, tidig uppväxling'),
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 3, 'Perfekta rutiner: speglar, blinkers, avsökning — inga missar på hela passet'),
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 4, 'Kort teorirepetition: skyltar och regler som känns osäkra (kunskapsprovet ska vara klart nu)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 1, 'Varför hänger sparsam körning och säker körning ihop? (båda handlar om samma sak: att läsa trafiken tidigt och planera)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 2, 'Nervositet inför provet: vad är elevens plan? (sömn, mat, komma i tid, våga be prövaren upprepa en instruktion — det är helt tillåtet)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 29), 3, 'Prövaren VILL godkänna — hur förändrar den tanken känslan inför provet?')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

INSERT INTO korschema_lessons (phase_id, ord, title, subtitle, goal, tip) VALUES
  ((SELECT id FROM korschema_phases WHERE ord = 6), 30, 'Generalrepetition i provorten', 'Sista passet före provet', 'Eleven är varm i kläderna i just de miljöer där provet körs och går in i provdagen lugn och förberedd.', 'Dagen före provet: kör INTE ett långt hårt pass. En kort lugn runda eller vila. Lycka till — ni har gjort jobbet! 🎉')
ON CONFLICT (ord) DO UPDATE SET phase_id = EXCLUDED.phase_id, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle, goal = EXCLUDED.goal, tip = EXCLUDED.tip;
INSERT INTO korschema_exercises (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 1, 'Kör i området runt Trafikverkets förarprovskontor — de miljötyper som finns där (rondeller? motorvägspåfart? stadskärna?)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 2, 'En sista lugn helrunda — ingen ny inlärning, bara bekräftelse'),
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 3, 'Snabb säkerhetskontroll som rutinkoll'),
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 4, 'Gå igenom provdagen praktiskt: tid, legitimation, bil, vad som händer steg för steg')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;
INSERT INTO korschema_discussion_topics (lesson_id, ord, text) VALUES
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 1, 'Vad är planen om det INTE blir godkänt? (helt odramatiskt: man får ett tydligt protokoll på vad som brast, tränar det och bokar om — många duktiga förare klarar det på andra försöket)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 2, 'Och efter godkänt: de första två åren har nya förare högst olycksrisk — vilka egna regler sätter eleven upp? (inga passagerare som stressar, aldrig trött, aldrig mobil)'),
  ((SELECT id FROM korschema_lessons WHERE ord = 30), 3, 'Sista frågan, den viktigaste: varför finns egentligen allt det här — provet, reglerna, träningen? (inte för att klara ett prov — för att alla ska komma hem)')
ON CONFLICT (lesson_id, ord) DO UPDATE SET text = EXCLUDED.text;

-- Milstolpar
INSERT INTO korschema_milestones (after_phase, icon, title, text) VALUES
  (3, '⚠️', 'Boka nu: Riskutbildning del 1 ("Riskettan")', 'Obligatorisk före provet och som bäst just här — teorin om alkohol, trötthet och grupptryck ger djup åt diskussionerna i Fas 4–6.'),
  (5, '🧊', 'Boka nu: Riskutbildning del 2 (halkbanan) + kunskapsprovet', 'Risk 2 kräver god körvana — perfekt läge i början av Fas 6. Kunskapsprovet ska vara godkänt innan körprovet kan göras, och giltighetstiden är begränsad — planera ihop dem.')
ON CONFLICT (after_phase) DO UPDATE SET icon = EXCLUDED.icon, title = EXCLUDED.title, text = EXCLUDED.text;

-- Statiska infosektioner. Keys and JSON shape are consumed as-is by the
-- frontend (types.ts: BeforeYouStart / Assessment) - keep the camelCase
-- keys 'beforeYouStart' / 'assessment' exactly.
INSERT INTO korschema_static_sections (key, content) VALUES
  ('beforeYouStart', '{"title":"Det formella — checklista för privat övningskörning","requirements":[{"title":"Körkortstillstånd","text":"Eleven ansöker hos Transportstyrelsen (hälsodeklaration + synintyg). Krävs innan första passet."},{"title":"Introduktionsutbildning","text":"Både handledare och elev måste ha gått den (giltig i 5 år). Görs på trafikskola."},{"title":"Godkänd handledare","text":"Handledaren ansöker om handledarskap hos Transportstyrelsen — ett godkännande per elev."},{"title":"Grön ÖVNINGSKÖR-skylt","text":"Bak på bilen vid varje pass. Kontrollera också att bilens försäkring gäller vid övningskörning."}],"beforeExam":["Riskutbildning del 1 (\"Riskettan\" — alkohol, droger, trötthet, riskbeteende). Gör den tidigt, gärna runt Fas 3 — den ger stoff till diskussionerna.","Riskutbildning del 2 (halkbanan). Kräver god körvana — lägg den i Fas 6, nära provet. Båda gäller i 5 år.","Kunskapsprovet görs först; godkänt kunskapsprov har begränsad giltighetstid (för närvarande 4 månader), så boka körprovet inom den tiden. Plugga teori parallellt med körningen hela vägen."]}'::jsonb)
ON CONFLICT (key) DO UPDATE SET content = EXCLUDED.content;
INSERT INTO korschema_static_sections (key, content) VALUES
  ('assessment', '{"title":"Det här bedömer förarprövaren på körprovet","intro":"Provet tar 45 minuter: säkerhetskontroll, eventuell särskild manövrering och körning i blandad trafik. Prövaren gör en helhetsbedömning — enstaka småfel fäller inte, men brister i något av dessa områden gör det:","areas":[{"title":"Hastighetsanpassning","text":"Rätt fart för situationen — inte bara under gränsen, utan anpassad till sikt, väglag och omgivning."},{"title":"Placering","text":"Rätt körfält och rätt placering i körfältet, särskilt inför sväng."},{"title":"Avsökning & riskmedvetenhet","text":"Blicken långt fram, speglar, döda vinkeln — och att eleven visar att hen ser riskerna."},{"title":"Samspel & flyt","text":"Tydlig, beslutsam körning som andra trafikanter kan läsa. Överdriven tvekan kan fälla."},{"title":"Regeltillämpning","text":"Väjningsregler, skyltar, vägmarkeringar — tillämpade i praktiken, inte bara i teorin."},{"title":"Sparsam körning","text":"Planerad körning med motorbroms och jämn fart bedöms också — mjuk körning är säker körning."}],"outro":"I provet ingår ofta självständig körning: att köra mot ett mål efter skyltar eller enkel vägbeskrivning. Det tränas särskilt i Fas 4–6."}'::jsonb)
ON CONFLICT (key) DO UPDATE SET content = EXCLUDED.content;

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (3, 'korschema_seed')
ON CONFLICT (version) DO NOTHING;
