import { readFileSync } from 'fs';
import { Options } from 'sequelize';

export interface Config {
    db: Options;
    port: number;
    host: string;
}

const configFilePath = 'turingarena.config.json';

const defaultConfig: Config = {
    db: {
        storage: ':memory:',
        dialect: 'sqlite',
    },
    port: 3000,
    host: 'localhost',
};

export function loadConfig(path?: string): Config {
    if (path === undefined) {
        return defaultConfig;
    }
    try {
        return JSON.parse(readFileSync(path ?? configFilePath).toString());
    } catch {
        console.warn('Cannot read config file: using defualt config');

        return defaultConfig;
    }
}
