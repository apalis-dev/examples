# poem-graphql-mysql

An example that incorporates `poem`, `async-graphql` and `mysql` as a backend

## Getting Started

You need to setup Mysql first. Docker should help:

```bash
docker run -d --name test-mysql -e MYSQL_ROOT_PASSWORD=strong_password -e MYSQL_DATABASE=test_db -p 3306:3306 mysql
```

## Running

```bash
DATABASE_URL=mysql://root:strong_password@localhost:3306/test_db cargo run
```

You can now visit http://0.0.0.0:3000 for the interface or use curl.

```bash
 curl 'http://0.0.0.0:3000/graphql' \
  -H 'Accept-Language: en-US,en;q=0.5' \
  -H 'Connection: keep-alive' \
  -H 'Sec-GPC: 1' \
  -H 'accept: application/json, multipart/mixed' \
  -H 'content-type: application/json' \
  --data-raw '{"query":"mutation {\n  pushJob(job: { id: \"1\"}) \n}"}' \
  --insecure
```
