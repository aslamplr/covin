import React from "react";
import { Route, Redirect, RouteProps } from "react-router-dom";
import { AuthState } from "@aws-amplify/ui-components";
import { useAuth } from "./ProvideAuth";

export default function PrivateRoute({
  children,
  ...rest
}: React.PropsWithChildren<RouteProps>) {
  const [authState, user] = useAuth();
  return (
    <Route
      {...rest}
      render={({ location }) =>
        authState === AuthState.SignedIn && user ? (
          children
        ) : (
          <Redirect
            to={{
              pathname: "/auth",
              state: { from: location },
            }}
          />
        )
      }
    />
  );
}
