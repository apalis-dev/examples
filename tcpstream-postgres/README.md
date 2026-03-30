# tcpstream-postgres

An example that incorporates `TcpStream`, `tokio::mpsc` and `apalis-postgres` as a backend

## Getting Started

```bash
docker run --name postgres-db -e POSTGRES_PASSWORD=mypassword postgres
```


## Running

```bash
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/postgres cargo run
```

You can now push jobs

```bash
echo '{"id":1}' | nc 127.0.0.1 5000
```
