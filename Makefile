.PHONY: gen
gen: generate-apis generate-schema format
	@:

.PHONY: generate-schema
generate-schema:
	@cargo codegen schema