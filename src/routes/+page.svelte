<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  
	import { Button } from '$lib/components/ui/button/index.js'

  let name = $state("");
  let greetMsg = $state("");
  let sysinfo = $state("");
  let nidaq_info = $state("");
  let ws_info = $state("");

  async function pyenv_sysinfo() {
    // Call the Rust function to get Python environment information
    const result = await invoke("get_pyenv_sysinfo");
    sysinfo = `Python + System Info: ${JSON.stringify(result)}`;
  }
  async function get_nidaq_sysinfo() {
    // Call the Rust function to get NI-DAQmx information
    const result = await invoke("get_nidaq_sysinfo");
    nidaq_info = `NI-DAQmx Info: ${JSON.stringify(result)}`;
  }

  async function check_ws_server(event: Event) {
    event.preventDefault();
    let pid = await invoke("get_ws_pid");
    try {
      // Connect to the WebSocket server
      const ws = new WebSocket('ws://localhost:21749');
      
      ws.onopen = () => {
        console.log('Connected to WebSocket server');
        // Send TEST OK message
        ws.send('TEST OK');
      };
      
      ws.onmessage = (event) => {
        console.log('Received message:', event.data);
        ws_info = `WebSocket Response: ${event.data} + PID: ${pid}`;
        // Close the connection after receiving the response
        ws.close();
      };
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        ws_info = 'WebSocket connection failed';
      };
      
      ws.onclose = () => {
        console.log('WebSocket connection closed');
      };
      
    } catch (error) {
      console.error('Error connecting to WebSocket:', error);
      ws_info = `WebSocket error: ${error}`;
    }
  }

</script>

<main class="h-full p-2">
  <form class="row" onsubmit={pyenv_sysinfo}>
    <Button type="submit">Get Sys Info</Button>
  </form>
  <p>{sysinfo}</p>

  <form class="row" onsubmit={get_nidaq_sysinfo}>
    <Button type="submit">Get NI-DAQmx Info</Button>
  </form>
  <p>{nidaq_info}</p>

  <!-- Check WS server via echo -->
  <form class="row" onsubmit={check_ws_server}>
    <Button type="submit">Check WS Server</Button>
  </form>
  <p>{ws_info}</p>

</main>
