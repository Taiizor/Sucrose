using Microsoft.CodeAnalysis.Testing;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Collections.Immutable;
using System.Threading;
using System.Threading.Tasks;

namespace WinUIEx.Analyzers.Test
{
    [TestClass]
    public class WinUIExAnalyzersUnitTest : BaseAnalyzersUnitTest<WinUIEx.Analyzers.WinUIExAlwaysNullAnalyzer, WinUIEx.Analyzers.WinUIExAnalyzersCodeFixProvider>
    {
        [TestMethod]
        public async Task AlwaysNull_Microsoft_UI_Xaml_Window_Current()
        {
            var testCode = @"
    using Microsoft.UI.Xaml;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var current = {|#0:Window.Current|};
            }
        }
    }";
            var expected = Diagnostic("WinUIEx1001").WithLocation(0).WithArguments("Microsoft.UI.Xaml.Window.Current");
            await VerifyAnalyzerAsync(testCode, expected);
        }

        [TestMethod]
        public async Task DependencyObject_Dispatcher()
        {
            var testCode = @"
    using Microsoft.UI.Xaml.Controls;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var border = new Border();
                var dispatcher = {|#0:border.Dispatcher|};
            }
        }
    }";
            var expected = Diagnostic("WinUIEx1002").WithLocation(0).WithArguments("Microsoft.UI.Xaml.Controls.Border.Dispatcher", "Border.DispatcherQueue");
            await VerifyAnalyzerAsync(testCode, expected);
        }

        [TestMethod]
        public async Task DependencyObject_Dispatcher_RunAsync()
        {
            var testCode = @"
    using Microsoft.UI.Xaml.Controls;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var border = new Border();
                var dispatcher = {|#0:border.Dispatcher|}.RunAsync(Windows.UI.Core.CoreDispatcherPriority.Normal, () => {});
            }
        }
    }";
            var expected = Diagnostic("WinUIEx1002").WithLocation(0).WithArguments("Microsoft.UI.Xaml.Controls.Border.Dispatcher", "Border.DispatcherQueue");
            await VerifyAnalyzerAsync(testCode, expected);
        }

        [TestMethod]
        public async Task Window_Dispatcher()
        {
            var testCode = @"
    using Microsoft.UI.Xaml;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var window = new Window();
                var dispatcher = {|#0:window.Dispatcher|};
            }
        }
    }";
            var expected = Diagnostic("WinUIEx1002").WithLocation(0).WithArguments("Microsoft.UI.Xaml.Window.Dispatcher", "Window.DispatcherQueue");
            await VerifyAnalyzerAsync(testCode, expected);
        }
        [TestMethod]
        public async Task CustomWindow_Dispatcher()
        {
            var testCode = @"
    using Microsoft.UI.Xaml;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var window = new MyWindow();
                var dispatcher = {|#0:window.Dispatcher|};
            }
        }
        class MyWindow : Window { }
    }";
            var expected = Diagnostic("WinUIEx1002").WithLocation(0).WithArguments("ConsoleApplication1.MyWindow.Dispatcher", "MyWindow.DispatcherQueue");
            await VerifyAnalyzerAsync(testCode, expected);
        }

        private static readonly Lazy<ReferenceAssemblies> _lazyNet60WinUI1_0 =
               new Lazy<ReferenceAssemblies>(() =>
               {
                   return new ReferenceAssemblies("net6.0-windows10.0.19041.0", new PackageIdentity("Microsoft.NETCore.App.Ref", "6.0.0"), System.IO.Path.Combine("ref", "net6.0"))
                            .AddPackages(ImmutableArray.Create(new PackageIdentity[] {
                                new PackageIdentity("Microsoft.Windows.SDK.NET.Ref", "10.0.19041.29"),
                                new PackageIdentity("Microsoft.WindowsAppSDK", "1.4.230913002") }));
               });

        [TestMethod]
        [Ignore] // TODO: Implement the code fixer
        public async Task CodeFix_Dispatcher_To_DispatcherQueue()
        {
            var testCode = @"
    using Microsoft.UI.Xaml.Controls;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var border = new Border();
                var dispatcher = {|#0:border.Dispatcher|};
            }
        }
    }";

            var fixtest = @"
    using Microsoft.UI.Xaml.Controls;
    namespace ConsoleApplication1
    {
        class MyClass
        {    
            public void MethodName()
            {
                var border = new Border();
                var dispatcher = {|#0:border.DispatcherQueue|};
            }
        }
    }";

            var expected = Diagnostic("WinUIEx1002").WithLocation(0).WithArguments("Microsoft.UI.Xaml.Controls.Border.Dispatcher", "Border.DispatcherQueue");
            await VerifyCodeFixAsync(testCode, expected, fixtest);
        }
    }
}
