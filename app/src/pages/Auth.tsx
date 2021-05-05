import React from "react";
import {
  AmplifyAuthenticator,
  AmplifySignIn,
  AmplifySignUp,
} from "@aws-amplify/ui-react";

export default function Auth() {
  return (
    <AmplifyAuthenticator usernameAlias="email">
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
}
