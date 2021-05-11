import React, { Fragment } from "react";
import { Disclosure, Menu, Transition } from "@headlessui/react";
import { MenuIcon, XIcon } from "@heroicons/react/outline";
import { withRouter, Link, RouteComponentProps } from "react-router-dom";
import { AmplifySignOut } from "@aws-amplify/ui-react";
import { AuthState } from "@aws-amplify/ui-components";
import { useAuth } from "./auth/ProvideAuth";

const profile = ["Sign out"];

interface NavItem {
  title: string;
  path: string;
}

interface Props {
  navigation: NavItem[];
}

export default withRouter(
  ({
    navigation,
    location: { pathname: currentPath },
  }: Props & RouteComponentProps) => {
    const [authState] = useAuth();
    const isAuthenticated = authState === AuthState.SignedIn;

    return (
      <Disclosure as="nav" className="bg-gray-800">
        {({ open }) => (
          <>
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex items-center justify-between h-16">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    📍 <span className="text-gray-200">COVIN</span>
                  </div>
                  <div className="hidden md:block">
                    <div className="ml-10 flex items-baseline space-x-4">
                      {navigation.map(({ title, path }) =>
                        path === currentPath ? (
                          <Fragment key={title}>
                            {/* Current: "bg-gray-900 text-white", Default: "text-gray-300 hover:bg-gray-700 hover:text-white" */}
                            <Link
                              to={path}
                              className="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium"
                            >
                              {title}
                            </Link>
                          </Fragment>
                        ) : (
                          <Link
                            key={title}
                            to={path}
                            className="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium"
                          >
                            {title}
                          </Link>
                        )
                      )}
                    </div>
                  </div>
                </div>
                {isAuthenticated && (
                  <div className="hidden md:block">
                    <div className="ml-4 flex items-center md:ml-6">
                      {/* Profile dropdown */}
                      <Menu as="div" className="ml-3 relative">
                        {({ open }) => (
                          <>
                            <div>
                              <Menu.Button className="max-w-xs bg-gray-800 rounded-full flex items-center text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white">
                                <span className="sr-only">Open user menu</span>
                                <img
                                  className="h-8 w-8 rounded-full"
                                  src="iu.png"
                                  alt=""
                                />
                              </Menu.Button>
                            </div>
                            <Transition
                              show={open}
                              as={Fragment}
                              enter="transition ease-out duration-100"
                              enterFrom="transform opacity-0 scale-95"
                              enterTo="transform opacity-100 scale-100"
                              leave="transition ease-in duration-75"
                              leaveFrom="transform opacity-100 scale-100"
                              leaveTo="transform opacity-0 scale-95"
                            >
                              <Menu.Items
                                static
                                className="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white ring-1 ring-black ring-opacity-5 focus:outline-none"
                              >
                                {profile.map((item) => (
                                  <Menu.Item key={item}>
                                    {({ active }) => (
                                      <React.Fragment key={item}>
                                        {isAuthenticated && <AmplifySignOut />}
                                      </React.Fragment>
                                    )}
                                  </Menu.Item>
                                ))}
                              </Menu.Items>
                            </Transition>
                          </>
                        )}
                      </Menu>
                    </div>
                  </div>
                )}
                <div className="-mr-2 flex md:hidden">
                  {/* Mobile menu button */}
                  <Disclosure.Button className="bg-gray-800 inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white">
                    <span className="sr-only">Open main menu</span>
                    {open ? (
                      <XIcon className="block h-6 w-6" aria-hidden="true" />
                    ) : (
                      <MenuIcon className="block h-6 w-6" aria-hidden="true" />
                    )}
                  </Disclosure.Button>
                </div>
              </div>
            </div>

            <Disclosure.Panel className="md:hidden">
              <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3">
                {navigation.map(({ title, path }) =>
                  path === currentPath ? (
                    <Fragment key={title}>
                      {/* Current: "bg-gray-900 text-white", Default: "text-gray-300 hover:bg-gray-700 hover:text-white" */}
                      <Link
                        to={path}
                        className="bg-gray-900 text-white block px-3 py-2 rounded-md text-base font-medium"
                      >
                        {title}
                      </Link>
                    </Fragment>
                  ) : (
                    <Link
                      key={title}
                      to={path}
                      className="text-gray-300 hover:bg-gray-700 hover:text-white block px-3 py-2 rounded-md text-base font-medium"
                    >
                      {title}
                    </Link>
                  )
                )}
              </div>
              {isAuthenticated && (
                <div className="pt-4 pb-3 border-t border-gray-700">
                  <div className="flex items-center px-5">
                    <div className="flex-shrink-0">
                      <img
                        className="h-10 w-10 rounded-full"
                        src="iu.png"
                        alt=""
                      />
                    </div>
                    {/* <div className="ml-3">
                    <div className="text-base font-medium leading-none text-white">
                      Tom Cook
                    </div>
                    <div className="text-sm font-medium leading-none text-gray-400">
                      tom@example.com
                    </div>
                  </div> */}
                  </div>
                  <div className="mt-3 px-2 space-y-1">
                    {profile.map((item) => (
                      <React.Fragment key={item}>
                        {isAuthenticated && <AmplifySignOut />}
                      </React.Fragment>
                    ))}
                  </div>
                </div>
              )}
            </Disclosure.Panel>
          </>
        )}
      </Disclosure>
    );
  }
);
