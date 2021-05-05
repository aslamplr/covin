import React from "react";
import {
  BrowserRouter as Router,
  Switch,
  Route,
} from "react-router-dom";
import * as auth from "./services/auth";
import { ProvideAuth } from "./components/auth/ProvideAuth";
import PrivateRoute from "./components/auth/PrivateRoute";
import Navigation from "./components/Navigation";
import Locator from "./pages/Locator";
import Auth from "./pages/Auth";
import Alerts from "./pages/Alerts";

auth.configure();

const navigation = [
  {
    title: "Locator",
    path: "/",
  },
  {
    title: "Alerts",
    path: "/alerts",
  },
];

export default function App() {
  

  return (
    <div>
      <ProvideAuth>
        <Router>
          <Navigation navigation={navigation} />
          <Switch>
            <Route path="/auth">
              <Auth />
            </Route>
            <PrivateRoute path="/alerts">
              <Alerts />
            </PrivateRoute>
            <Route path="/">
              <Locator />
            </Route>
          </Switch>
        </Router>
      </ProvideAuth>
    </div>
  );
}
