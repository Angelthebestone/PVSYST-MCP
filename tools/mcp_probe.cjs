const base = 'C:/Users/Lenovo/AppData/Roaming/npm/node_modules/exa-mcp-server/node_modules/@modelcontextprotocol/sdk/dist/cjs/client';
const { Client } = require(base + '/index.js');
const { StdioClientTransport } = require(base + '/stdio.js');

(async () => {
  const transport = new StdioClientTransport({
    command: 'C:\\Users\\Lenovo\\Desktop\\PVSYST MCP\\target\\release\\pvsyst-mcp.exe',
    env: { ...process.env, PVSYST_WORKSPACE: 'C:\\Users\\Lenovo\\PVsyst8.1_Data' },
  });
  const client = new Client({ name: 'probe', version: '1.0.0' }, { capabilities: {} });
  await client.connect(transport);
  console.log('negotiated protocol:', client.getNegotiatedProtocolVersion && client.getNegotiatedProtocolVersion());
  console.log('server version:', JSON.stringify(client.getServerVersion && client.getServerVersion()));
  const tools = await client.listTools();
  console.log('tools:', tools.tools.length);
  const r = await client.callTool({ name: 'info_entorno', arguments: {} });
  console.log('callTool isError:', r.isError);
  if (!r.isError) {
    const t = JSON.parse(r.content[0].text);
    console.log('tool_count:', t.tool_count, 'workspace:', t.workspace);
  }
  await client.close();
  process.exit(0);
})().catch((e) => { console.error('FAIL:', e && (e.stack || e.message)); process.exit(1); });
