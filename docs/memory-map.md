## OS Memory Map

| Name   | Start        | End          | Size        | Flags                  | Description                                            |
|--------|--------------|--------------|-------------|------------------------|--------------------------------------------------------|
| LAPIC  | 500Gi        | 500Gi + 4KiB | 4KiB        | Ring0, RW, Passthrough | LAPIC memory mapping                                   |
| IOAPIC | 500Gi + 4KiB | 500Gi + 1MiB | 1MiB - 4KiB | Ring0, RW, Passthrough | IOAPIC mappings, base addr = `500Gi + 4KiB * (id + 1)` |
| HPET   | 502Gi        | 502Gi + 4KiB | 4KiB        | Ring0, RW, Passthrough | HPET mapping                                           |
| PCIe   | 512Gi        | 1TiB         | 512Gi       | Ring0, RW, Passthrough | PCIe mapping                                           |
|        |              |              |             |                        |                                                        |
