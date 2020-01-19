import { gql } from 'apollo-server-core';
import { Resolvers } from '../generated/graphql-types';
import { Contest, contestResolvers, contestSchema } from './contest';
import { ContestFile } from './contest-file';
import { ContestProblem } from './contest-problem';
import { Evaluation, EvaluationEvent } from './evaluation';
import { FileContent, fileSchema } from './file-content';
import { mutationResolvers, mutationSchema } from './mutation';
import { Participation } from './participation';
import { Problem, problemResolvers, problemSchema } from './problem';
import {
    ProblemFile,
    problemFileResolvers,
    problemFileSchema,
} from './problem-file';
import { queryResolvers, querySchema } from './query';
import { Submission, submissionSchema } from './submission';
import { SubmissionFile } from './submission-file';
import { User, userResolvers, userSchema } from './user';

/** Full GraphQL schema document. Obtained combining schema parts from each components. */
export const schema = gql`
    ${querySchema}
    ${mutationSchema}
    ${userSchema}
    ${contestSchema}
    ${problemSchema}
    ${problemFileSchema}
    ${fileSchema}
    ${submissionSchema}

    enum Ok {
        OK
    }
`;

/** All model classes constructors. */
export const modelConstructors = {
    User,
    Contest,
    Problem,
    ContestProblem,
    Participation,
    FileContent,
    ProblemFile,
    ContestFile,
    Submission,
    SubmissionFile,
    Evaluation,
    EvaluationEvent,
};

/** All GraphQL resolvers. Obtained combining resolvers from each components. */
export const resolvers: Resolvers = {
    ...queryResolvers,
    ...mutationResolvers,
    ...userResolvers,
    ...contestResolvers,
    ...problemResolvers,
    ...problemFileResolvers,
};