import type { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
	schema: "../brume/internal/router/public-gql/graph/public.graphql",
	documents: ["src/**/*.{ts,tsx}"],
	generates: {
		"./src/_apollo/": {
			preset: "client",
			presetConfig: {
				gqlTagName: "gql",
				inlineFragmentTypes: "inline",
				fragmentMasking: false,
			},
			config: {
				useTypeImports: true,
			},
		},
	},
	ignoreNoDocuments: true,
};

export default config;
