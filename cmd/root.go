package cmd

import (
	"fmt"
	"os"
	"time"

	"github.com/spf13/cobra"
	"github.com/theckman/yacspin"
	"savvyuni.com/checker/stages"
)

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "checker",
	Short: "Assignment Checker",
	Long:  fmt.Sprintf("Assignment Checker (SavvyUni & SavvyPro © 2014–%d)", time.Now().Year()),
	Run: func(cmd *cobra.Command, args []string) {
		phone, _ := cmd.Flags().GetString("phone")
		testId, _ := cmd.Flags().GetString("test_id")
		machine := stages.IdentifyMachine()
		spinner, _ := yacspin.New(yacspin.Config{
			Frequency:         200 * time.Millisecond,
			Colors:            []string{"fgYellow"},
			ColorAll:          true,
			CharSet:           yacspin.CharSets[78],
			Suffix:            " Checker",
			SuffixAutoColon:   true,
			StopCharacter:     "✓",
			StopColors:        []string{"fgGreen"},
			StopMessage:       "All tests finished",
			StopFailCharacter: "✗",
			StopFailColors:    []string{"fgRed"},
		})
		spinner.Start()
		spinner.Message("validating files..")
		for _, filePath := range args {
			if err := stages.CheckFileCriteria(filePath); err != nil {
				spinner.StopFailMessage(err.Error())
				_ = spinner.StopFail()
				os.Exit(1)
			}
		}
		spinner.Message("authorizing...")
		instruction, err := stages.CanISumbit(machine, testId, phone)
		if err != nil {
			spinner.StopFailMessage(err.Error())
			_ = spinner.StopFail()
			os.Exit(1)
		}
		if instruction.SubmissionId == "" {
			spinner.StopFailMessage(instruction.Message)
			_ = spinner.StopFail()
			os.Exit(1)
		}

		spinner.Message("submitting...")
		var files []stages.OSSFile
		for _, filePath := range args {
			updatedFile, err := stages.UploadFile(instruction, filePath)
			if err != nil {
				spinner.StopFailMessage("Could not submit answers. Please check your network connection.")
				_ = spinner.StopFail()
				os.Exit(1)
			}
			files = append(files, *updatedFile)
		}

		if len(files) == 0 {
			spinner.StopMessage("No files available, program exists without running tests.")
			_ = spinner.Stop()
		}

		spinner.Message("judging...")
		testBody := stages.RunBody{
			TestEntry: instruction.TestEntry,
			TestEnv:   instruction.TestEnv,
			Files:     files,
		}
		testResult, err := stages.RunTest(instruction.RunnerLocation, &testBody)
		result := testResult
		if err != nil {
			result += err.Error()
		}
		resultBody := stages.TestResult{
			SubmissionId: instruction.SubmissionId,
			Result:       result,
			Files:        files,
		}
		stages.SendAnalytics(&resultBody)
		spinner.Stop()
	},
}

func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	rootCmd.PersistentFlags().StringP("phone", "p", "", "Phone number to claim your identify")
	rootCmd.PersistentFlags().StringP("test_id", "i", "", "Test set ID")
	rootCmd.MarkFlagRequired("phone")
	rootCmd.MarkFlagRequired("test_id")
}
