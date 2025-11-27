# NoSQL Rust Frontend

React frontend for the NoSQL Rust Monitoring System.

## Quick Start

### Local Development

```bash
npm install
npm start
```

The app will run at `http://localhost:3000`

### Docker Development

```bash
docker build -f frontend/Dockerfile -t nosql-frontend .
docker run -p 3000:3000 -e REACT_APP_API_URL=http://localhost:3000 nosql-frontend
```

### Docker Compose

```bash
# With database services
docker compose -f docker-compose.yaml up -d
docker compose -f docker-compose-frontend.yaml up -d

# Or combined
docker compose -f docker-compose.yaml -f docker-compose-frontend.yaml up -d
```

## Environment Variables

- `REACT_APP_API_URL` - Backend API URL (default: http://localhost:3000)

Copy `.env.example` to `.env.local` and adjust if needed.

## Build for Production

```bash
npm run build
```

This creates an optimized production build in the `build/` folder.
