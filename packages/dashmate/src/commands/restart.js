import { Listr } from 'listr2';

import { Flags } from '@oclif/core';
import ConfigBaseCommand from '../oclif/command/ConfigBaseCommand.js';
import MuteOneLineError from '../oclif/errors/MuteOneLineError.js';

export default class RestartCommand extends ConfigBaseCommand {
  static description = 'Restart node';

  static flags = {
    ...ConfigBaseCommand.flags,
    platform: Flags.boolean({ char: 'p', description: 'restart only platform', default: false }),
    safe: Flags.boolean({ char: 's', description: 'wait for dkg before stop', default: false }),
  };

  /**
   * @param {Object} flags
   * @param {restartNodeTask} restartNodeTask
   * @param {Config} config
   * @return {Promise<void>}
   */
  async runWithDependencies({
    flags: {
      verbose: isVerbose,
      platform: platformOnly,
      safe: isSafe,
    },
    restartNodeTask,
    config,
  }) {
    const tasks = new Listr(
      [
        {
          title: `Restarting ${config.getName()} node`,
          task: () => restartNodeTask(config),
        },
      ],
      {
        renderer: isVerbose ? 'verbose' : 'default',
        rendererOptions: {
          showTimer: isVerbose,
          clearOutput: false,
          collapse: false,
          showSubtasks: true,
        },
      },
    );

    try {
      await tasks.run({
        isVerbose,
        isSafe,
        platformOnly: platformOnly === true,
      });
    } catch (e) {
      throw new MuteOneLineError(e);
    }
  }
}
