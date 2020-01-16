import { gql } from 'apollo-server-core';
import { AutoIncrement, Column, Index, Model, NotNull, PrimaryKey, Table, Unique } from 'sequelize-typescript';

export const fileSchema = gql`
    type File {
        hash: ID!
        fileName: String!
        type: String!
        contentBase64: String!
    }

    input FileInput {
        hash: ID!
        fileName: String!
        type: String!
        contentBase64: String!
    }
`;

@Table({ updatedAt: false })
export class File extends Model<File> {
    @Unique
    @Index
    @Column
    hash!: string;

    @Column
    type!: string;

    @Column
    fileName!: string;

    @Column
    content!: Buffer;
}
