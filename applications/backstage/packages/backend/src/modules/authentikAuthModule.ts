import { createBackendModule } from '@backstage/backend-plugin-api';
import {
  authProvidersExtensionPoint,
  createOAuthProviderFactory,
} from '@backstage/plugin-auth-node';
import { oidcAuthenticator } from '@backstage/plugin-auth-backend-module-oidc-provider';
import { stringifyEntityRef, DEFAULT_NAMESPACE } from '@backstage/catalog-model';

export const authentikAuthModule = createBackendModule({
  pluginId: 'auth',
  moduleId: 'authentik-auth-provider',
  register(reg) {
    reg.registerInit({
      deps: { providers: authProvidersExtensionPoint },
      async init({ providers }) {
        providers.registerProvider({
          providerId: 'oidc',
          factory: createOAuthProviderFactory({
            authenticator: oidcAuthenticator,
            async signInResolver(info, ctx) {
              const { profile } = info;

              if (!profile.email) {
                throw new Error('User profile does not contain an email');
              }

              // Create user entity reference from email
              // Extract local part of email as username (e.g., bo.kristoffersson from bo.kristoffersson@me.com)
              const emailLocalPart = profile.email.split('@')[0];

              const userEntityRef = stringifyEntityRef({
                kind: 'User',
                name: emailLocalPart,
                namespace: DEFAULT_NAMESPACE,
              });

              return ctx.issueToken({
                claims: {
                  sub: userEntityRef,
                  ent: [userEntityRef],
                },
              });
            },
          }),
        });
      },
    });
  },
});
