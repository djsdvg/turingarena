import { Injectable } from '@angular/core';
import { Query } from 'apollo-angular';
import gql from 'graphql-tag';
import { SubmissionListQuery, SubmissionListQueryVariables } from './__generated__/SubmissionListQuery';
import { problemMaterialFragment } from './graphql-fragments';

@Injectable({
  providedIn: 'root'
})
export class SubmissionListQueryService extends Query<SubmissionListQuery, SubmissionListQueryVariables> {
  document = gql`
    query SubmissionListQuery($userId: String!, $problemName: ProblemName!) {
      user(id: $userId) {
        id
        problem(name: $problemName) {
          name
          submissions {
            id
            createdAt
            files {
              fieldId
              typeId
              name
              contentBase64
            }
            status
            scores {
              scorableId
              score
            }
          }
          ...ProblemMaterialFragment
        }
      }
    }
    ${problemMaterialFragment}
  `;
}
