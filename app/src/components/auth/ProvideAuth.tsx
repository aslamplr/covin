import React, { createContext, useContext } from "react";
import { AuthState, onAuthUIStateChange } from "@aws-amplify/ui-components";

interface AuthData {
  attributes: {
    email: string;
    email_verified: boolean;
    name: string;
    phone_number: string;
    phone_number_verified: boolean;
    sub: string;
  }
}

const authContext = createContext<[AuthState | undefined, AuthData | undefined]>([
  AuthState.Loading,
  undefined,
]);

export function ProvideAuth({ children }: React.PropsWithChildren<{}>) {
  const [authState, setAuthState] = React.useState<AuthState>();
  const [user, setUser] = React.useState<AuthData | undefined>();

  React.useEffect(() => {
    return onAuthUIStateChange((nextAuthState, authData) => {
      setAuthState(nextAuthState);
      setUser(authData as AuthData);
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
