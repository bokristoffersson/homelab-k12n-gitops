import { createBackendModule, coreServices } from '@backstage/backend-plugin-api';
import { createRouter } from '@backstage-community/plugin-kafka-backend';

export default createBackendModule({
  pluginId: 'kafka',
  moduleId: 'kafka',
  register(reg) {
    reg.registerInit({
      deps: {
        httpRouter: coreServices.httpRouter,
        config: coreServices.rootConfig,
        logger: coreServices.logger,
      },
      async init({ httpRouter, config, logger }) {
        const router = await createRouter({
          logger,
          config,
        });
        httpRouter.use(router);
      },
    });
  },
});

