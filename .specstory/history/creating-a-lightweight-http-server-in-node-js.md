## SpecStory

## Creating a Lightweight HTTP Server in Node.js (2/14/2025, 10:29:06 PM)

_**User**_

create a lightweight http server using nodejs native libraries

---

_**Assistant**_

I'll help you create a lightweight HTTP server using Node.js's built-in `http` module. I'll modify your `index.js` file to implement this.

Let me edit the file to create a basic HTTP server:

---

_****_

```diff
- console.log("Hello, world!");
+ const http = require('http');
+ 
+ const PORT = process.env.PORT || 3000;
+ 
+ const server = http.createServer((req, res) => {
+     // Set CORS headers
+     res.setHeader('Access-Control-Allow-Origin', '*');
+     res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
+     res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
+ 
+     // Handle preflight requests
+     if (req.method === 'OPTIONS') {
+         res.writeHead(204);
+         res.end();
+         return;
+     }
+ 
+     // Set default content type
+     res.setHeader('Content-Type', 'application/json');
+ 
+     // Basic routing
+     if (req.url === '/') {
+         res.writeHead(200);
+         res.end(JSON.stringify({
+             message: 'Welcome to the Node.js HTTP Server',
+             status: 'running'
+         }));
+     } else {
+         res.writeHead(404);
+         res.end(JSON.stringify({
+             error: 'Not Found',
+             message: 'The requested resource was not found'
+         }));
+     }
+ });
+ 
+ server.listen(PORT, () => {
+     console.log(`Server is running on http://localhost:${PORT}`);
+ });
+ 
+ // Error handling
+ server.on('error', (error) => {
+     if (error.code === 'EADDRINUSE') {
+         console.error(`Port ${PORT} is already in use`);
+     } else {
+         console.error('Server error:', error);
+     }
+     process.exit(1);
+ });
+ 
+ // Graceful shutdown
+ process.on('SIGTERM', () => {
+     console.log('Received SIGTERM signal. Shutting down gracefully...');
+     server.close(() => {
+         console.log('Server closed');
+         process.exit(0);
+     });
+ });
+ 
+ process.on('SIGINT', () => {
+     console.log('Received SIGINT signal. Shutting down gracefully...');
+     server.close(() => {
+         console.log('Server closed');
+         process.exit(0);
+     });
+ });
```

---

_****_

I've created a lightweight HTTP server using Node.js's native `http` module. Here are the key features of this implementation:

1. **Port Configuration**: The server runs on port 3000 by default, but you can change it using the `PORT` environment variable.

2. **CORS Support**: Built-in CORS headers to allow cross-origin requests, which is helpful for development and API usage.

3. **Basic Routing**:
   - `/` endpoint returns a welcome message
   - All other routes return a 404 error

4. **Error Handling**:
   - Handles server errors (like port already in use)
   - Includes graceful shutdown handlers for SIGTERM and SIGINT signals

To run the server:

1. Open your terminal
2. Navigate to the directory containing `index.js`
3. Run: `node index.js`

The server will start and listen on port 3000 (or your specified PORT environment variable). You can test it by:

- Opening `http://localhost:3000` in your browser
- Or using curl: `curl http://localhost:3000`

You'll get a JSON response with a welcome message for the root endpoint, and a 404 error message for any other endpoints.

Would you like me to add any additional endpoints or features to the server?

---

---
