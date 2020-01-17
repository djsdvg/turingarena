import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'yaml';
import * as mime from 'mime-types';
import { ApiContext } from '../api/index';
import { UserPrivilege } from '../model/user';

/**
 * Walk the directory and inserts all the files in the database
 *
 * @param fileList a list of File
 * @param base directory to walk
 * @param ctx API context
 */
async function addFiles(fileList: object[], base: string, dir: string = '') {
    const files = fs.readdirSync(path.join(base, dir));
    for (const file of files) {
        const relPath = path.join(dir, file);
        if (fs.statSync(path.join(base, relPath)).isDirectory()) {
            await addFiles(fileList, base, relPath);
        } else {
            const content = fs.readFileSync(path.join(base, relPath));
            const type = mime.lookup(file);
            fileList.push({
                path: relPath,
                type: type !== false ? type : 'unknown',
                content,
            });
        }
    }
}

/**
 * Import a contest in the database
 *
 * @param dir base directoyr of the contest
 */
export async function _import(dir = process.cwd()) {
    const ctx = new ApiContext();

    await ctx.sequelize.sync();

    const turingarenaYAMLPath = path.join(dir, 'turingarena.yaml');

    if (!fs.existsSync(turingarenaYAMLPath))
        throw Error('Invalid contest directory');

    const turingarenaYAML = fs.readFileSync(turingarenaYAMLPath).toString();
    const contest = yaml.parse(turingarenaYAML);

    contest.files = [];
    await addFiles(contest.files, path.join(dir, 'files'));

    console.info('Importing contest', contest);

    await ctx.db.Contest.create(contest, { include: [ctx.db.File] });

    for (const user of contest.users) {
        await ctx.db.User.create({
            username: user.username,
            name: user.name,
            token: user.token,
            privilege: user.role === 'admin' ? UserPrivilege.ADMIN : UserPrivilege.USER,
        });
    }

    for (const problem of contest.problems) {
        const toInsert = {
            name: problem,
            files: [],
        }

        await addFiles(toInsert.files, path.join(dir, problem));

        await ctx.db.Problem.create(toInsert, { include: [ctx.db.File] });
    }
}
