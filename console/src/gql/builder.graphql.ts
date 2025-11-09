import { gql } from "@/_apollo/gql";

export const BUILDER_FRAGMENT = gql(`
  fragment BuilderFragment on Builder {
    id
    type
    data
    link
    version
    schema
  }
`);
