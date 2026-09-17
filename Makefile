.PHONY: gen
gen: generate-schema
	@:

.PHONY: generate-schema
generate-schema:
	@cargo codegen schema