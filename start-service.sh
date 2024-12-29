#!/bin/bash
bash -c "exec -a bobby3 python3 test.py &>> bobby3.log &" # bobby3
bash -c "exec -a bobby python3 test.py &>> bobby.log &" # bobby
bash -c "exec -a bobby2 python3 test.py &>> bobby2.log &" # bobby2
bash -c "exec -a bobby4 python3 test.py &>> bobby4.log &" # bobby4
bash -c "exec -a bobby5 python3 test.py &>> bobby5.log &" # bobby5