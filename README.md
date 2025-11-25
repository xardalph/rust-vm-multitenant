# nosql-rust

This school project objective is to make a simple multitenant monitoring system with postgres for client configuration, victoriaMetrics for metrics storage, and maybe redis for session cache.
the objective is to have a user create an "agent" in the webapp (added in the postgresql db)
create the agent as a docker container, which will authenticate and send data to a rust backend, which will store it in one victoria metrics for every client using victoria metrics cluster tenant functionality

## Installation

REQUIRE UNIX SOCKET

The agent require docker unix socket to be accessible (windows is implemented but not tested.). If you don't have an unix socket use a virtual machine for a proper linux.
If you are on mac, you should check your docker socket location and fix the docker compose files accordingly.

Clone the repository, and run docker compose up -d with diferent parameter depending on your situation :

### dev - only DB in docker

start every Database with :

```sh
docker compose  -d
```

Then run the rust server part with :

```sh
RUST_LOG=debug cargo run --bin server
```

and start the agent with :

```sh
RUST_LOG=debug cargo run --bin agent
```

### Prod - no traefik
You want to build all the code automaticaly and run VM, postgres, redis, the rust backend, the agent in a container each :
```sh
docker compose -f docker-compose.yaml -f docker-compose-rust.yaml up -d
```

Access webapp on http://localhost:3000

### Prod - with traefik
TODO. Need to add label at least on the rust server to have https serving, and probably 2 container of the same server to be fun.
```sh
docker compose -f docker-compose.yaml -f docker-compose-rust.yaml up -d
```

## technical architecture

VM (victoria metric) is used to store every metrics sent.
There is 2 rust binary :
- agent read a docker socket and send data about each container to a defined url every second per container (use a token to authenticate)
- server accept agent trafic and redirect it to VM.  \
  It also accept web user to be able to manage agent configuration, and get data from VM. \
  server is multitenant, which mean one agent configured for a company won't be seen by another company. \
  For now company is a flat list, there is no parent and children.

Redis is used for session storage (implemented by rust crate axum_login)

postgresql is used to store company, user and agent configuration.


### api connexion

You can see the frontend at the root of the webapp, by default http://localhost:3000/

To develop the frontend, you should use the api provided.

Authenticate by post on `/login` url :
```
curl localhost:3000/login -d 'username=usernameHere&password=YourPassword' -v
```
Then retrieve the cookie "id" from the response, and put this cookie in all subsequent api endpoint to be authenticated.

be carefull that when you try to use the api without authentication you will get a 307 to /login page.
## Roadmap


### Victoria Metrics

this show how to push data to VM : https://docs.victoriametrics.com/victoriametrics/#json-line-format
to send metrics and retrieve it with curl :
```
echo '{"metric":{"__name__":"evan_metric1","job":"curl","instance":"vmagent:8429"},"values":[100,300],"timestamps":[1763074402660,1763074402661]}'  > /tmp/vmFile.json
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
