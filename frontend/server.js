const express = require('express');
const path = require('path');
const app = express();
const port = process.env.PORT || 3001;

app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));
app.get('/health', (req, res) => res.json({ project: 'AssetDesk', status: 'running' }));

app.listen(port, () => {
  console.log(`AssetDesk frontend is running on http://localhost:${port}`);
});
