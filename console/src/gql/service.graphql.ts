import { gql } from "@/_apollo/gql";

export const BASE_SERVICE_FRAGMENT = gql(`
  fragment BaseServiceFragment on BaseService {
    source {
      ...SourceFragment
    }
    runner {
      ...RunnerFragment
    }
    builder {
      ...BuilderFragment
    }
  }
`);

export const ServiceFragment = gql(`
  fragment ServiceFragment on Service {
      name
      id
      live {
        ...BaseServiceFragment
      }
      draft {
        ...BaseServiceFragment
      }
   }
`);
