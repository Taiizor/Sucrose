## WebAuthenticator

The WebAuthenticator helps you simplify OAuth workflows in your application with a simple single-line call.

First in your application constructor or in the Main program, add an OAuth startup activation check:
```cs
public App()
{
    if (WebAuthenticator.CheckOAuthRedirectionActivation())
        return;
    this.InitializeComponent();
}
```

Next you can make a make a call to authenticate using your default browser:
```cs
WebAuthenticatorResult result = await WinUIEx.WebAuthenticator.AuthenticateAsync(authorizeUrl, callbackUri);
```

Your callback uri must use a custom scheme, and you must define this scheme in your application manifest. For example if your callback uri is "myscheme://loggedin", your manifest dialog should look like this:

![image](https://user-images.githubusercontent.com/1378165/166501267-1da07930-ab4d-431e-87cf-a7b183cc3c87.png)


