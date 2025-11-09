import { gql } from "@/_apollo/gql";

// export const ADD_SERVICE_MUTATION = gql(`
//   mutation AddServiceToProject(
//     $projectId: String!
//     $input: CreateServiceInput!
//   ) {
//     addServiceToProject(projectId: $projectId, input: $input) {
//       id
//       name
//     }
//   }
// `);

// export const DELETE_SERVICE_MUTATION = gql(`
//   mutation DeleteService($serviceId: String!) {
//     deleteService(serviceId: $serviceId) {
//       id
//     }
//   }
// `);

// export const UPDATE_SERVICE_SETTINGS_MUTATION = gql(`
//   mutation UpdateServiceSettings(
//     $serviceId: String!
//     $input: ServiceSettingsInput!
//   ) {
//     updateServiceSettings(serviceId: $serviceId, input: $input) {
//       id
//       name
//     }
//   }
// `);

export const BaseServiceFragment = gql(`
  fragment BaseServiceFragment on BaseService {
    source {
      type
      data
    }
    runner {
      type
      link
      version
      schema
      data
    }
    builder {
      type
      link
      version
      schema
      data
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
