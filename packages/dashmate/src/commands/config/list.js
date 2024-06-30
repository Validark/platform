import { table } from 'table';
import BaseCommand from '../../oclif/command/BaseCommand.js';

export default class ConfigListCommand extends BaseCommand {
  static description = 'List available configs';

  /**
   * @param {ConfigFile} configFile
   * @return {Promise<void>}
   */
  async runWithDependencies({
    configFile,
  }) {
    const rows = configFile.getAllConfigs()
      .map((config) => [config.getName(), config.get('description')]);

    const output = table(rows);

    // eslint-disable-next-line no-console
    console.log(output);
  }
}
