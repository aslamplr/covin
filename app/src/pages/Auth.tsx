import React from "react";
import {
  AmplifyAuthenticator,
  AmplifySignIn,
  AmplifySignUp,
} from "@aws-amplify/ui-react";
import { AuthState } from "@aws-amplify/ui-components";
import { withRouter, RouteComponentProps } from "react-router-dom";

export default withRouter(({history, location}: RouteComponentProps) => {
  const handleAuthStateChange = (nextAuthState: AuthState) => {
    if (nextAuthState === AuthState.SignedIn) {
      let redirectPath = "/";
      const locationState = location.state as { from: { pathname: string }};
      if (locationState && locationState.from) {
        redirectPath = locationState.from.pathname;
      }
      history.push(redirectPath);
    }
  };
  return (
    <AmplifyAuthenticator
      usernameAlias="email"
      handleAuthStateChange={handleAuthStateChange}
    >
      <AmplifySignUp
        slot="sign-up"
        usernameAlias="email"
        formFields={[
          {
            type: "name",
            label: "Name",
            placeholder: "Enter your name",
            required: true,
          },
          {
            type: "email",
            label: "Email",
            placeholder: "Enter email address",
            required: true,
          },
          {
            type: "password",
            label: "Password",
            placeholder: "Enter a strong password",
            required: true,
          },
          {
            type: "phone_number",
            label: "Phone",
            placeholder: "Phone number",
            required: false,
          },
        ]}
      />
      <AmplifySignIn slot="sign-in" usernameAlias="email" />
    </AmplifyAuthenticator>
  );
});
