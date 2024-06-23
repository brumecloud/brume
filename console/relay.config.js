export default {
  src: "./src",
  schema: "./src/__generated__/graphql.ts",
  exclude: ["**/node_modules/**", "**/__mocks__/**", "**/__generated__/**"],
  extensions: ["ts", "tsx"],
  artifactDirectory: "./src/__generated__",
  language: "typescript",
  eagerEsModules: true,
  customScalars: {
    DateTime: "string",
  },
};