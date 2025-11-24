# Grafana Dashboard för Värmepump

Detta dokument beskriver Grafana-konfigurationen för visualisering av värmepumpdata från TimescaleDB.

## Översikt

Dashboarden visar följande data från värmepumpen:

### Temperaturer
- **Utetemperatur** - Temperatur utanför huset
- **Framledning** - Temperatur på framledningen till värmesystemet
- **Returledning** - Temperatur på returledningen från värmesystemet
- **Varmvatten** - Temperatur på varmvattnet
- **Brine ut** - Temperatur på brine-vätskan ut från värmepumpen
- **Brine in** - Temperatur på brine-vätskan in till värmepumpen

### Pumphastigheter
- **Framledningspump** - Hastighet på framledningspumpen (%)
- **Brinepump** - Hastighet på brinepumpen (%)

### Körningstider
- **Kompressor** - Total körtid för kompressorn (sekunder)
- **Varmvatten** - Total körtid för varmvattenproduktion (sekunder)
- **3kW** - Körtid för 3kW-elementet (sekunder)
- **6kW** - Körtid för 6kW-elementet (sekunder)

### Komponentstatus
- **Brinepump** - Om brinepumpen är på/av
- **Kompressor** - Om kompressorn är på/av
- **Framledningspump** - Om framledningspumpen är på/av
- **Varmvatten** - Om varmvattenproduktion är aktiv
- **Cirkulationspump** - Om cirkulationspumpen är på/av

### Integral
- **Integral** - Regleringsintegral från värmepumpen

## Konfiguration

### Datasource
Dashboarden använder TimescaleDB som datasource med följande inställningar:
- **Typ**: PostgreSQL
- **URL**: `timescaledb.heatpump-mqtt.svc.cluster.local:5432`
- **Databas**: `timescaledb`
- **Användare**: `timescaledb`

### Dashboard-inställningar
- **Uppdateringsintervall**: 30 sekunder
- **Standard tidsintervall**: Senaste timmen
- **Tidszon**: Systemstandard

## Deployment

Dashboarden deployas automatiskt via GitOps när följande filer uppdateras:

1. `datasource-postgres.yaml` - PostgreSQL datasource-konfiguration
2. `dashboard-heatpump.yaml` - Dashboard JSON-konfiguration
3. `grafana-postgres-secret-sealed.yaml` - Lösenord för databasanslutning
4. `helmrelease.yaml` - Grafana Helm-konfiguration

## Användning

1. Logga in på Grafana
2. Navigera till "Dashboards" i sidomenyn
3. Välj "Värmepump Dashboard"
4. Använd tidsväljaren för att ändra tidsintervall
5. Klicka på panelerna för att zooma in på specifika tidsperioder

## Felsökning

### Datasource-problem
Om dashboarden inte visar data:
1. Kontrollera att TimescaleDB är tillgänglig
2. Verifiera att lösenordet i `grafana-postgres-secret-sealed.yaml` är korrekt
3. Kontrollera att värmepump-MQTT-applikationen skriver data till databasen

### Dashboard-problem
Om dashboarden inte laddas:
1. Kontrollera att ConfigMap `grafana-dashboard-heatpump` är skapad
2. Verifiera att dashboard JSON är giltig
3. Kontrollera Grafana-pod loggar för felmeddelanden

## Anpassning

Dashboarden kan anpassas genom att redigera JSON-konfigurationen i `dashboard-heatpump.yaml`. Vanliga ändringar:

- Lägga till nya paneler
- Ändra tidsintervall
- Modifiera färgscheman
- Lägga till alerting-regler

Efter ändringar, commit och push till GitOps-repot för att deploya ändringarna.
