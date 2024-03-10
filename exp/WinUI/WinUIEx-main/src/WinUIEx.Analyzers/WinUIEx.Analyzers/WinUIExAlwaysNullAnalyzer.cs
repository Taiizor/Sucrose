using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using System.Collections.Generic;
using System.Collections.Immutable;

namespace WinUIEx.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class WinUIExAlwaysNullAnalyzer : DiagnosticAnalyzer
    {
        static Dictionary<string, string> AlwaysNullProperties = new Dictionary<string, string>() {
            { "Microsoft.UI.Xaml.Window.Current", null },
        };

        public const string DiagnosticId1001 = "WinUIEx1001";
        public const string DiagnosticId1002 = "WinUIEx1002";

        // You can change these strings in the Resources.resx file. If you do not want your analyzer to be localize-able, you can use regular strings for Title and MessageFormat.
        // See https://github.com/dotnet/roslyn/blob/main/docs/analyzers/Localizing%20Analyzers.md for more on localization
        private static readonly LocalizableString Title1001 = new LocalizableResourceString(nameof(Resources.AlwaysNullTitle), Resources.ResourceManager, typeof(Resources));
        private static readonly LocalizableString MessageFormat1001 = new LocalizableResourceString(nameof(Resources.AlwaysNullMessageFormat), Resources.ResourceManager, typeof(Resources));
        private static readonly LocalizableString Description1001 = new LocalizableResourceString(nameof(Resources.AlwaysNullDescription), Resources.ResourceManager, typeof(Resources));
        private static readonly LocalizableString Title1002 = new LocalizableResourceString(nameof(Resources.DispatcherTitle), Resources.ResourceManager, typeof(Resources));
        private static readonly LocalizableString MessageFormat1002 = new LocalizableResourceString(nameof(Resources.DispatcherMessageFormat), Resources.ResourceManager, typeof(Resources));
        private static readonly LocalizableString Description1002 = new LocalizableResourceString(nameof(Resources.DispatcherDescription), Resources.ResourceManager, typeof(Resources));
        private const string Category = "Usage";

        private static readonly DiagnosticDescriptor AlwaysNullRule = new DiagnosticDescriptor(DiagnosticId1001, Title1001, MessageFormat1001, Category, DiagnosticSeverity.Warning, isEnabledByDefault: true, description: Description1001, helpLinkUri: "https://dotmorten.github.io/WinUIEx/rules/WinUIEx1001.html");
        private static readonly DiagnosticDescriptor DispatcherRule = new DiagnosticDescriptor(DiagnosticId1002, Title1002, MessageFormat1002, Category, DiagnosticSeverity.Warning, isEnabledByDefault: true, description: Description1002, helpLinkUri: "https://dotmorten.github.io/WinUIEx/rules/WinUIEx1002.html");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics { get { return ImmutableArray.Create(AlwaysNullRule, DispatcherRule); } }

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterOperationAction(AnalyzeOperation, OperationKind.PropertyReference);
            //context.RegisterSyntaxNodeAction(AnalyzeExpression, SyntaxKind.InvocationExpression);

        }

        //private void AnalyzeExpression(SyntaxNodeAnalysisContext context)
        //{
        //    var invocationExpr = (InvocationExpressionSyntax)context.Node;
        //    var memberAccessExpr = invocationExpr.Expression as MemberAccessExpressionSyntax;
        //    if (memberAccessExpr == null)
        //        return;
        //    var memberSymbol = context.SemanticModel.GetSymbolInfo(memberAccessExpr).Symbol;
        //    if(memberSymbol.ContainingType.ToDisplayString() +"." + memberSymbol.Name == "Windows.UI.Core.CoreDispatcher.RunAsync")
        //    {
        //        var diagnostic = Diagnostic.Create(RuleWithAlternative, memberAccessExpr.GetLocation(), "Windows.UI.Core.CoreDispatcher.RunAsync", "DispatcherQueue.TryEnqueueAsync");
        //        context.ReportDiagnostic(diagnostic);
        //    }
        //}

        private void AnalyzeOperation(OperationAnalysisContext context)
        {
            Diagnostic diagnostic = null;
            var propRefOp = context.Operation as Microsoft.CodeAnalysis.Operations.IPropertyReferenceOperation;
            if (propRefOp != null)
            {
                var namedTypeSymbol = propRefOp.Member.ToDisplayString();
                var instanceNamedTypeSymbol = propRefOp.Instance != null ? (propRefOp.Instance.Type.ToDisplayString() + "." + propRefOp.Member.Name) : namedTypeSymbol;

                if (namedTypeSymbol == "Microsoft.UI.Xaml.DependencyObject.Dispatcher" || namedTypeSymbol == "Microsoft.UI.Xaml.Window.Dispatcher")
                {
                    diagnostic = Diagnostic.Create(DispatcherRule, propRefOp.Syntax.GetLocation(), instanceNamedTypeSymbol, propRefOp.Instance.Type.Name + ".DispatcherQueue");
                }
                else if (AlwaysNullProperties.ContainsKey(namedTypeSymbol))
                {
                    diagnostic = Diagnostic.Create(AlwaysNullRule, propRefOp.Syntax.GetLocation(), instanceNamedTypeSymbol);
                }
            }
            if (diagnostic != null)
                context.ReportDiagnostic(diagnostic);
        }
    }
}