using Microsoft.CodeAnalysis.CodeFixes;
using Microsoft.CodeAnalysis.Diagnostics;
using Microsoft.CodeAnalysis.Testing;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Collections.Immutable;
using System.Threading;
using System.Threading.Tasks;

namespace WinUIEx.Analyzers.Test
{
    public abstract class BaseAnalyzersUnitTest<TAnalyzer, TCodeFix>
        where TAnalyzer : DiagnosticAnalyzer, new()
        where TCodeFix : CodeFixProvider, new()
    {
        private static readonly Lazy<ReferenceAssemblies> _lazyNet60WinUI1_0 =
               new Lazy<ReferenceAssemblies>(() =>
               {
                   return new ReferenceAssemblies("net6.0-windows10.0.19041.0", new PackageIdentity("Microsoft.NETCore.App.Ref", "6.0.0"), System.IO.Path.Combine("ref", "net6.0"))
                            .AddPackages(ImmutableArray.Create(new PackageIdentity[] {
                                new PackageIdentity("Microsoft.Windows.SDK.NET.Ref", "10.0.19041.29"),
                                new PackageIdentity("Microsoft.WindowsAppSDK", "1.0.0") }));
               });
        public static ReferenceAssemblies Net60WinUI1_0 => _lazyNet60WinUI1_0.Value;

        protected static async Task VerifyAnalyzerAsync(string source, params DiagnosticResult[] expected)
        {
            var test = new CSharpCodeFixVerifier<TAnalyzer,TCodeFix>.Test
            {
                ReferenceAssemblies = Net60WinUI1_0,
                TestCode = source,
            };
            test.ExpectedDiagnostics.AddRange(expected);
            await test.RunAsync(CancellationToken.None);
        }
        protected static async Task VerifyCodeFixAsync(string source, DiagnosticResult expected, string fixedSource)
        {
            var test = new CSharpCodeFixVerifier<TAnalyzer, TCodeFix>.Test
            {
                ReferenceAssemblies = Net60WinUI1_0,
                TestCode = source,
                FixedCode = fixedSource
            };
            test.ExpectedDiagnostics.Add(expected);
            await test.RunAsync(CancellationToken.None);
        }
        protected DiagnosticResult Diagnostic(string diagnosticId) => WinUIEx.Analyzers.Test.CSharpCodeFixVerifier<TAnalyzer, TCodeFix>.Diagnostic(diagnosticId);
    }
}
