


.PHONY: all test clean

all: test

examples/output.json:
	gh search code --extension "edn" --json "repository,url" --limit 800 > examples/output.json

examples/output/%: examples/output.json
	rm -r examples/output
	mkdir -p examples/output
	jq -r '.[].url' < examples/output.json | sed -e "s/github/raw.githubusercontent/" | sed -e "s/\/blob//" | xargs wget -P examples/output

valid_edn.txt invalid_edn.txt: examples/output/%
	cargo test tests::test_valid_edn_files -- --ignored --nocapture

test: valid_edn.txt invalid_edn.txt
	cargo test

clean:
	rm -r examples/output
