package stages

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/denisbrodbeck/machineid"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/host"
	"github.com/shirou/gopsutil/v3/mem"
)

func CheckFileCriteria(filePath string) error {
	fileInfo, err := os.Stat(filePath)
	if os.IsNotExist(err) {
		return fmt.Errorf("file does not exist: %s", filePath)
	}
	if err != nil {
		return fmt.Errorf("error checking file: %s", err)
	}

	const maxFileSize = 1 << 20 // 1MB in bytes
	if fileInfo.Size() > maxFileSize {
		return fmt.Errorf("file size exceeds 1MB: %s", filePath)
	}

	return nil
}

type NetworkInterface struct {
	Name    string `json:"name"`
	Address string `json:"address"`
}

type Machine struct {
	ID                string             `json:"id"`
	Hostname          string             `json:"hostname"`
	Platform          string             `json:"platform"`
	CpuModel          string             `json:"cpuModel"`
	CpuCores          int32              `json:"cpuCores"`
	MemGB             float64            `json:"memGB"`
	NetworkInterfaces []NetworkInterface `json:"networkInterfaces"`
}

func IdentifyMachine() Machine {
	m := Machine{}

	if hostInfo, err := host.Info(); err == nil {
		m.Hostname = hostInfo.Hostname
		m.Platform = fmt.Sprintf("%s %s", hostInfo.Platform, hostInfo.PlatformVersion)
	}

	if cpuInfo, err := cpu.Info(); err == nil {
		for _, info := range cpuInfo {
			m.CpuModel = info.ModelName
			m.CpuCores = info.Cores
		}
	}

	if memInfo, err := mem.VirtualMemory(); err == nil {
		totalMemGB := float64(memInfo.Total) / float64(1<<30) // Convert bytes to GB
		m.MemGB = totalMemGB
	}

	if id, err := machineid.ProtectedID("savvyuni.com/checker"); err == nil {
		m.ID = id
	}

	if interfaces, err := macAddress(); err == nil {
		m.NetworkInterfaces = interfaces
	}

	return m
}

func macAddress() ([]NetworkInterface, error) {
	interfaces, err := net.Interfaces()
	if err != nil {
		return nil, err
	}

	var networkInterfaces []NetworkInterface

	for _, iface := range interfaces {
		// Check if the interface is up and not a loopback
		if iface.Flags&net.FlagUp != 0 && iface.Flags&net.FlagLoopback == 0 && iface.HardwareAddr.String() != "" {
			networkInterfaces = append(networkInterfaces, NetworkInterface{
				Name:    iface.Name,
				Address: iface.HardwareAddr.String(),
			})
		}
	}
	return networkInterfaces, nil
}

type Payload struct {
	SetId   string  `json:"setId"`
	Phone   string  `json:"phone"`
	Machine Machine `json:"machine"`
}

type SubmissionInstruction struct {
	Message        string    `json:"message"`
	SubmissionId   string    `json:"SubmissionId"`
	Bucket         string    `json:"Bucket"`
	Dir            string    `json:"Dir"`
	OSSAccessKeyId string    `json:"OSSAccessKeyId"`
	Policy         string    `json:"Policy"`
	Signature      string    `json:"Signature"`
	RunnerLocation string    `json:"RunnerLocation"`
	TestEntry      string    `json:"TestEntry"`
	TestEnv        []OSSFile `json:"TestEnv"`
}

func CanISumbit(m Machine, testId string, phone string) (*SubmissionInstruction, error) {
	payload := Payload{
		SetId:   testId,
		Phone:   phone,
		Machine: m,
	}
	jsonData, _ := json.Marshal(payload)
	req, err := http.NewRequest("POST", AUTH_ENDPOINT, bytes.NewReader(jsonData))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")

	client := http.Client{
		Timeout: 30 * time.Second,
	}

	res, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf(strings.ReplaceAll(err.Error(), AUTH_ENDPOINT, "auth server"))
	}
	defer res.Body.Close()

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("Failed to read submission instructions")
	}

	var instruction SubmissionInstruction
	err = json.Unmarshal(body, &instruction)
	if err != nil {
		return nil, fmt.Errorf("Failed to interpret submission instructions")
	}

	return &instruction, err
}
