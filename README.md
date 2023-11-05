# portal-testground


### Run a test
Terminal 1
```bash
testground daemon
```
Terminal 2
```bash
git clone https://github.com/KolbyML/portal-testground.git
testground plan import --from ./portal-testground
testground run single --plan=portal-testground --testcase=ping-one-way --runner=local:docker --builder=docker:generic --instances=2 --wait
```

### How to set client for test
Terminal 2
```bash
testground run single --plan=portal-testground --testcase=ping --runner=local:docker \
 --builder=docker:generic --instances=2 --wait \
 --test-param="client1=fluffy" --test-param="client2=trin"
```
