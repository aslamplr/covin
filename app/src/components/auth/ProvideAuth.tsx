import React, { createContext, useContext } from "react";
import { AuthState, onAuthUIStateChange } from "@aws-amplify/ui-components";

const authContext = createContext<[AuthState | undefined, object | undefined]>([
  AuthState.Loading,
  undefined,
]);

export function ProvideAuth({ children }: React.PropsWithChildren<{}>) {
  const [authState, setAuthState] = React.useState<AuthState>();
  const [user, setUser] = React.useState<object | undefined>();

  React.useEffect(() => {
    return onAuthUIStateChange((nextAuthState, authData) => {
      setAuthState(nextAuthState);
      setUser(authData);
    });
  }, []);

  return (
    <authContext.Provider value={[authState, user]}>
      {children}
    </authContext.Provider>
  );
}

export function useAuth() {
  return useContext(authContext);
}
