# Dirac

## Run Dirac

`dirac> RUST_BACKTRACE=1 PYTHONPATH=../check_modules cargo run -- ../examples/pdt.yml -r markdown -o ../reports/pdt-report.md`

## Run Tests

### Run All Module Tests

`check_modules> nosetests modules`

### Run Specific Module Test

`check_modules> nosetests modules/ssh_tests.py`


