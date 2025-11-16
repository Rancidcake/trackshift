# API Integration Guide

## Backend Connection

To connect to your Rust backend, update the API routes in `/app/api/`:

### 1. Update `/app/api/metrics/route.ts`

Replace the simulated data with actual backend calls:

```typescript
export async function GET() {
  const response = await fetch('http://localhost:8080/api/metrics', {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN' }
  })
  const data = await response.json()
  return NextResponse.json(data)
}
```

### 2. Update `/app/api/upload/route.ts`

Connect to your Rust file transfer service:

```typescript
export async function POST(request: NextRequest) {
  const formData = await request.formData()
  const file = formData.get('file') as File
  
  // Send to Rust backend
  const response = await fetch('http://localhost:8080/api/upload', {
    method: 'POST',
    body: formData,
  })
  
  return NextResponse.json(await response.json())
}
```

### 3. WebSocket for Real-Time Updates

For real-time updates, add WebSocket support:

```typescript
// In components, use WebSocket
useEffect(() => {
  const ws = new WebSocket('ws://localhost:8080/ws')
  
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data)
    // Update component state
  }
  
  return () => ws.close()
}, [])
```

## Expected API Endpoints

Your Rust backend should provide:

- `GET /api/metrics` - Current system metrics
- `POST /api/upload` - File upload
- `POST /api/transfer/start` - Start file transfer
- `GET /api/transfer/status` - Transfer status
- `WebSocket /ws` - Real-time updates

## Data Format

### Metrics Response:
```json
{
  "transfer": {
    "fileName": "file.zip",
    "fileSize": 104857600,
    "bytesTransferred": 45678912,
    "speed": 12.5,
    "compressionRatio": 0.65,
    "pqcHandshake": true,
    "estimatedTime": 45,
    "status": "active",
    "latency": 15.2
  },
  "paths": [...],
  "fec": {...},
  "integrity": {...}
}
```

