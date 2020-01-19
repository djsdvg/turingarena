import { DataTypes } from 'sequelize';
import { AllowNull, BelongsTo, Column, ForeignKey, Model, Table } from 'sequelize-typescript';
import { Evaluation } from './evaluation';

/** Evant of an evaluation */
@Table({ updatedAt: false })
export class EvaluationEvent extends Model<EvaluationEvent> {
    @ForeignKey(() => Evaluation)
    @AllowNull(false)
    @Column
    evaluationId!: number;

    /** Evaluation to which this event belongs to */
    @BelongsTo(() => Evaluation, 'evaluationId')
    evaluation!: Evaluation;

    /** Data of this event, in a backend-specific format */
    @AllowNull(false)
    @Column(DataTypes.JSON)
    data!: TaskMakerEvent;
}

type TaskMakerEvent =
    | TaskMakerIOISolutionEvent
    | TaskMakerIOISubtaskScoreEvent
    | TaskMakerIOITestCaseScoreEvent
    | TaskMakerCompilationEvent;

interface TaskMakerExecutionResult {
    status: string;
    was_killed: boolean;
    was_cached: boolean;
    resources: {
        cpu_time: number;
        sys_time: number;
        wall_time: number;
        memory: number;
    };
}

// FIXME: should this be an enum or union type?
interface TaskMakerStatus {
    Done: {
        result: TaskMakerExecutionResult;
    };
}

interface TaskMakerIOISolutionEvent {
    IOIEvaluation: {
        subtask: number;
        testcase: number;
        solution: string;
        status: TaskMakerStatus;
    };
}

interface TaskMakerIOITestCaseScoreEvent {
    IOITestcaseScore: {
        subtask: number;
        testcase: number;
        solution: string;
        score: number;
        message: string;
    };
}

interface TaskMakerIOISubtaskScoreEvent {
    IOISubtaskScore: {
        subtask: number;
        solution: string;
        normalized_score: number;
        score: number;
    };
}

interface TaskMakerCompilationEvent {
    Compilation: {
        file: string;
        status: TaskMakerStatus;
    };
}
