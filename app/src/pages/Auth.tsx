import React from "react";
import {
  AmplifyAuthenticator,
  AmplifySignIn,
  AmplifySignUp,
} from "@aws-amplify/ui-react";
import { AuthState } from "@aws-amplify/ui-components";
import { withRouter, RouteComponentProps } from "react-router-dom";

export default withRouter(({ history, location }: RouteComponentProps) => {
  React.useEffect(() => {
    // This is a hacky way to remove the rest of the country code from the the
    // Amplify sign up component! not robust!!
    // There are no option to limit country code in the component interface
    // Or else we need to create a custom sign-up component!
    // So, hack for the time being!! 😅
    setTimeout(() => {
      const countryCodeSelect = document
        .querySelector("amplify-sign-up")
        ?.shadowRoot?.querySelector("amplify-auth-fields")
        ?.shadowRoot?.querySelector("amplify-phone-field")
        ?.shadowRoot?.querySelector("amplify-form-field")
        ?.querySelector("amplify-country-dial-code")
        ?.shadowRoot?.querySelector("amplify-select")
        ?.shadowRoot?.querySelector("select");
      for (let i = countryCodeSelect?.children.length!; i >= 0; i--) {
        const option = countryCodeSelect?.children[i];
        if (option?.textContent !== "+91") {
          option?.remove();
        }
      }
    }, 2000);
  }, []);

  const handleAuthStateChange = (nextAuthState: AuthState) => {
    if (nextAuthState === AuthState.SignedIn) {
      let redirectPath = "/";
      const locationState = location.state as { from: { pathname: string } };
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
