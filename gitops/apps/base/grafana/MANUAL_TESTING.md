# Workflow för GitOps-integration efter manuell testning

## Efter manuell testning av dashboard

### 1. Exportera dashboard JSON
1. I Grafana: Dashboard Settings → JSON Model
2. Kopiera hela JSON-koden
3. Ersätt innehållet i `dashboard-heatpump.yaml` under `data.heatpump-dashboard.json`

### 2. Uppdatera datasource-konfiguration (om ändringar gjorts)
Om du ändrat datasource-inställningar manuellt:
1. Gå till Configuration → Data Sources → TimescaleDB
2. Kopiera inställningarna
3. Uppdatera `datasource-postgres.yaml` eller `helmrelease.yaml`

### 3. Commita ändringar
```bash
git add apps/base/grafana/
git commit -m "feat: uppdatera värmepump dashboard efter manuell testning"
git push
```

### 4. Verifiera deployment
```bash
# Kontrollera att ConfigMaps skapas korrekt
kubectl get configmaps -n grafana | grep grafana

# Kontrollera att dashboard laddas
kubectl logs -n grafana deployment/grafana | grep dashboard
```

### 5. Testa slutlig dashboard
1. Logga in på Grafana
2. Kontrollera att dashboarden finns under "Dashboards"
3. Verifiera att alla paneler visar data korrekt
4. Testa tidsintervall-väljaren och zoom-funktioner

## Tips för manuell testning

### Bra queries att testa:
- **Senaste data**: `WHERE ts > NOW() - INTERVAL '1 hour'`
- **Daglig sammanfattning**: Använd `time_bucket('1 hour', ts)` för timvis aggregering
- **Min/Max värden**: `MIN(outdoor_temp), MAX(outdoor_temp)`
- **Genomsnitt**: `AVG(supplyline_temp)`

### Visualiseringstyper att testa:
- **Time Series** - för temperaturer och hastigheter
- **State Timeline** - för boolean-statusar
- **Stat** - för aktuella värden
- **Gauge** - för procentuella värden
- **Bar Chart** - för runtime-statistik

### Färgscheman:
- **Temperature**: Blå-röd gradient
- **Pumphastigheter**: Grön-gul-röd
- **Status**: Grön (på) / Röd (av)
- **Runtime**: Blå toningar
