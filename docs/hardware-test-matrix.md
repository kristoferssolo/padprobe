# Hardware test matrix

Keep manual observations factual. Record the connection path, whether Steam was
running, and the backend-visible behavior. Do not infer physical failure from
mapped input alone.

| Scenario | Enumeration | Buttons/axes | Hotplug | Rumble | Notes |
| --- | --- | --- | --- | --- | --- |
| Host-reported “Microsoft Xbox 360” controller | Pass | Not tested | Not tested | Not tested | `gilrs` reported SDL mapping, wired power, and VID:PID `045e:028e`; clean `q` exit verified |
| USB controller | Not tested | Not tested | Not tested | Not tested | |
| Bluetooth controller | Not tested | Not tested | Not tested | Not tested | |
| PlayStation-style controller | Not tested | Not tested | Not tested | Not tested | |
| Nintendo-style controller | Not tested | Not tested | Not tested | Not tested | |
| Generic USB controller | Not tested | Not tested | Not tested | Not tested | |
| Multiple controllers | Not tested | Not tested | Not tested | Not tested | |
| Steam running | Not tested | Not tested | Not tested | Not tested | Check for virtual-controller duplication |
| Steam fully closed | Not tested | Not tested | Not tested | Not tested | Compare mapping and device count |
| Controller disconnect while selected | Not tested | Not tested | Not tested | Not tested | Expected: retain disconnected selection |
| SSH terminal | Not tested | Not tested | Not tested | Not tested | Verify resize and terminal restoration |

## Suggested test record

For each manual run, capture:

- PadProbe version or commit
- Kernel and distribution
- Terminal and session type (local/SSH, Wayland/X11 where relevant)
- Controller name and connection path
- Steam/Input-layer state
- Controls observed and any unknown mappings
- Hotplug result
- Rumble result, including an unsupported response
- Whether normal exit, `Ctrl-C`, and an induced panic restore the terminal
