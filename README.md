# nosql-rust

This school project objective is to make a simple multitenant monitoring system with postgres for client configuration, victoriaMetrics for metrics storage, and maybe redis for session cache.
the objective is to have a user create an "agent" in the webapp (added in the postgresql db)
create the agent as a docker container, which will authenticate and send data to a rust backend, which will store it in one victoria metrics for every client


## Roadmap

### Victoria Metrics

this show how to push data to VM : https://docs.victoriametrics.com/victoriametrics/#json-line-format
to send metrics and retrieve it with curl :
```
echo '{"metric":{"__name__":"evan-metric1","job":"curl","instance":"vmagent:8429"},"values":[100,300],"timestamps":[1763074402660,1763074402661]}'  > /tmp/vmFile.json
curl -H 'Content-Type: application/json' --data-binary "@/tmp/vmFile.json" -X POST http://localhost:8480/insert/0/prometheus/api/v1/import

curl http://localhost:8428/api/v1/export -d 'match={__name__="evan-metric1"}'
```
output :
```
{"metric":{"__name__":"evan-metric1","job":"curl","instance":"vmagent:8429"},"values":[100,300],"timestamps":[1763074402660,1763074402661]}
```

for docker stats : https://docs.rs/docker-api/latest/docker_api/api/container/struct.Container.html#method.stats

### Axum/rest api


show an exemple of redis implementation for the sessions management, and may be a good idea for easier route management : https://github.com/AlexandreBarbier/axum_router_helper/tree/main/axum-rh

Seem to be the lib for redis session : https://github.com/maxcountryman/tower-sessions-stores/tree/main/redis-store

for database, sqlx can have concrete db type (impossible to change later) or "Any" which can be any db type, but the macro sqlx::query_as! won't work with it.
You can use this to retrieve a struct "Agent" from the db : `sqlx::query_as::<_, Agent>("select * from agent where token = $1").bind(token).fetch_all(&db).await?;`



## Authors and acknowledgment
Evan ADAM
Alexei KADIR
Héloise BOUSSON

## License
AGPLV3

## Project status
School project
