# trin-testground


### Run a test
Terminal 1
```bash
testground daemon
```
Terminal 2
```bash
git clone https://github.com/KolbyML/trin-testground.git
testground plan import --from ./trin-testground
testground run single --plan=trin-testground --testcase=publish-subscribe --runner=local:docker --builder=docker:generic --instances=2 --wait
```