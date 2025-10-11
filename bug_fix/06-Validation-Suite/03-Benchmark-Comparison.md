| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/prtip -s connect -p 22,53,80 -T 2 scanme.nmap.org` | 3.006 ± 0.002 | 3.004 | 3.009 | 1.73 ± 0.01 |
| `nmap -sT -T3 -p 22,53,80 scanme.nmap.org` | 1.736 ± 0.015 | 1.715 | 1.754 | 1.00 |
| `nmap -sT -T4 -p 22,53,80 scanme.nmap.org` | 1.754 ± 0.030 | 1.720 | 1.797 | 1.01 ± 0.02 |
