import { gql } from "@/_apollo/gql";

export const SourceFragment = gql(`
  fragment SourceFragment on Source {
    id
    type
    data
  }
`);
