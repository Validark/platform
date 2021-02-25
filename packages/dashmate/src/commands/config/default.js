const BaseCommand = require('../../oclif/command/BaseCommand');

class ConfigDefaultCommand extends BaseCommand {
  /**
   * @param {Object} args
   * @param {Object} flags
   * @param {ConfigFile} configFile
   * @return {Promise<void>}
   */
  async runWithDependencies(
    {
      config: configName,
    },
    flags,
    configFile,
  ) {
    if (configName === null) {
      // eslint-disable-next-line no-console
      console.log(configFile.getDefaultConfigName());
    } else {
      configFile.setDefaultConfigName(configName);

      // eslint-disable-next-line no-console
      console.log(`${configName} config set as default`);
    }
  }
}

ConfigDefaultCommand.description = `Manage default config

Shows default config name or sets another config as default
`;

ConfigDefaultCommand.args = [{
  name: 'config',
  required: false,
  description: 'config name',
  default: null,
}];

module.exports = ConfigDefaultCommand;
